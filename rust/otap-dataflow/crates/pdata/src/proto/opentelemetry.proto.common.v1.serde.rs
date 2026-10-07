impl serde::Serialize for AnyValue {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if self.value.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.common.v1.AnyValue", len)?;
        if let Some(v) = self.value.as_ref() {
            match v {
                any_value::Value::StringValue(v) => {
                    struct_ser.serialize_field("stringValue", v)?;
                }
                any_value::Value::BoolValue(v) => {
                    struct_ser.serialize_field("boolValue", v)?;
                }
                any_value::Value::IntValue(v) => {
                    #[allow(clippy::needless_borrow)]
                    #[allow(clippy::needless_borrows_for_generic_args)]
                    struct_ser.serialize_field("intValue", ToString::to_string(&v).as_str())?;
                }
                any_value::Value::DoubleValue(v) => {
                    struct_ser.serialize_field("doubleValue", v)?;
                }
                any_value::Value::ArrayValue(v) => {
                    struct_ser.serialize_field("arrayValue", v)?;
                }
                any_value::Value::KvlistValue(v) => {
                    struct_ser.serialize_field("kvlistValue", v)?;
                }
                any_value::Value::BytesValue(v) => {
                    #[allow(clippy::needless_borrow)]
                    #[allow(clippy::needless_borrows_for_generic_args)]
                    struct_ser.serialize_field("bytesValue", pbjson::private::base64::encode(&v).as_str())?;
                }
            }
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for AnyValue {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "string_value",
            "stringValue",
            "bool_value",
            "boolValue",
            "int_value",
            "intValue",
            "double_value",
            "doubleValue",
            "array_value",
            "arrayValue",
            "kvlist_value",
            "kvlistValue",
            "bytes_value",
            "bytesValue",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            StringValue,
            BoolValue,
            IntValue,
            DoubleValue,
            ArrayValue,
            KvlistValue,
            BytesValue,
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
                            "stringValue" | "string_value" => Ok(GeneratedField::StringValue),
                            "boolValue" | "bool_value" => Ok(GeneratedField::BoolValue),
                            "intValue" | "int_value" => Ok(GeneratedField::IntValue),
                            "doubleValue" | "double_value" => Ok(GeneratedField::DoubleValue),
                            "arrayValue" | "array_value" => Ok(GeneratedField::ArrayValue),
                            "kvlistValue" | "kvlist_value" => Ok(GeneratedField::KvlistValue),
                            "bytesValue" | "bytes_value" => Ok(GeneratedField::BytesValue),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = AnyValue;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.common.v1.AnyValue")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<AnyValue, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut value__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::StringValue => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("stringValue"));
                            }
                            value__ = map_.next_value::<::std::option::Option<_>>()?.map(any_value::Value::StringValue);
                        }
                        GeneratedField::BoolValue => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("boolValue"));
                            }
                            value__ = map_.next_value::<::std::option::Option<_>>()?.map(any_value::Value::BoolValue);
                        }
                        GeneratedField::IntValue => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("intValue"));
                            }
                            value__ = map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| any_value::Value::IntValue(x.0));
                        }
                        GeneratedField::DoubleValue => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("doubleValue"));
                            }
                            value__ = map_.next_value::<::std::option::Option<::pbjson::private::NumberDeserialize<_>>>()?.map(|x| any_value::Value::DoubleValue(x.0));
                        }
                        GeneratedField::ArrayValue => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("arrayValue"));
                            }
                            value__ = map_.next_value::<::std::option::Option<_>>()?.map(any_value::Value::ArrayValue)
;
                        }
                        GeneratedField::KvlistValue => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("kvlistValue"));
                            }
                            value__ = map_.next_value::<::std::option::Option<_>>()?.map(any_value::Value::KvlistValue)
;
                        }
                        GeneratedField::BytesValue => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("bytesValue"));
                            }
                            value__ = map_.next_value::<::std::option::Option<::pbjson::private::BytesDeserialize<_>>>()?.map(|x| any_value::Value::BytesValue(x.0));
                        }
                    }
                }
                Ok(AnyValue {
                    value: value__,
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.common.v1.AnyValue", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for ArrayValue {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.values.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.common.v1.ArrayValue", len)?;
        if !self.values.is_empty() {
            struct_ser.serialize_field("values", &self.values)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for ArrayValue {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "values",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Values,
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
                            "values" => Ok(GeneratedField::Values),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = ArrayValue;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.common.v1.ArrayValue")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<ArrayValue, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut values__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Values => {
                            if values__.is_some() {
                                return Err(serde::de::Error::duplicate_field("values"));
                            }
                            values__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(ArrayValue {
                    values: values__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.common.v1.ArrayValue", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for EntityRef {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.schema_url.is_empty() {
            len += 1;
        }
        if !self.r#type.is_empty() {
            len += 1;
        }
        if !self.id_keys.is_empty() {
            len += 1;
        }
        if !self.description_keys.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.common.v1.EntityRef", len)?;
        if !self.schema_url.is_empty() {
            struct_ser.serialize_field("schemaUrl", &self.schema_url)?;
        }
        if !self.r#type.is_empty() {
            struct_ser.serialize_field("type", &self.r#type)?;
        }
        if !self.id_keys.is_empty() {
            struct_ser.serialize_field("idKeys", &self.id_keys)?;
        }
        if !self.description_keys.is_empty() {
            struct_ser.serialize_field("descriptionKeys", &self.description_keys)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for EntityRef {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "schema_url",
            "schemaUrl",
            "type",
            "id_keys",
            "idKeys",
            "description_keys",
            "descriptionKeys",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            SchemaUrl,
            Type,
            IdKeys,
            DescriptionKeys,
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
                            "schemaUrl" | "schema_url" => Ok(GeneratedField::SchemaUrl),
                            "type" => Ok(GeneratedField::Type),
                            "idKeys" | "id_keys" => Ok(GeneratedField::IdKeys),
                            "descriptionKeys" | "description_keys" => Ok(GeneratedField::DescriptionKeys),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = EntityRef;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.common.v1.EntityRef")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<EntityRef, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut schema_url__ = None;
                let mut r#type__ = None;
                let mut id_keys__ = None;
                let mut description_keys__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::SchemaUrl => {
                            if schema_url__.is_some() {
                                return Err(serde::de::Error::duplicate_field("schemaUrl"));
                            }
                            schema_url__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Type => {
                            if r#type__.is_some() {
                                return Err(serde::de::Error::duplicate_field("type"));
                            }
                            r#type__ = Some(map_.next_value()?);
                        }
                        GeneratedField::IdKeys => {
                            if id_keys__.is_some() {
                                return Err(serde::de::Error::duplicate_field("idKeys"));
                            }
                            id_keys__ = Some(map_.next_value()?);
                        }
                        GeneratedField::DescriptionKeys => {
                            if description_keys__.is_some() {
                                return Err(serde::de::Error::duplicate_field("descriptionKeys"));
                            }
                            description_keys__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(EntityRef {
                    schema_url: schema_url__.unwrap_or_default(),
                    r#type: r#type__.unwrap_or_default(),
                    id_keys: id_keys__.unwrap_or_default(),
                    description_keys: description_keys__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.common.v1.EntityRef", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for InstrumentationScope {
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
        if !self.version.is_empty() {
            len += 1;
        }
        if !self.attributes.is_empty() {
            len += 1;
        }
        if self.dropped_attributes_count != 0 {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.common.v1.InstrumentationScope", len)?;
        if !self.name.is_empty() {
            struct_ser.serialize_field("name", &self.name)?;
        }
        if !self.version.is_empty() {
            struct_ser.serialize_field("version", &self.version)?;
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
impl<'de> serde::Deserialize<'de> for InstrumentationScope {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "name",
            "version",
            "attributes",
            "dropped_attributes_count",
            "droppedAttributesCount",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Name,
            Version,
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
                            "name" => Ok(GeneratedField::Name),
                            "version" => Ok(GeneratedField::Version),
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
            type Value = InstrumentationScope;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.common.v1.InstrumentationScope")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<InstrumentationScope, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut name__ = None;
                let mut version__ = None;
                let mut attributes__ = None;
                let mut dropped_attributes_count__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Name => {
                            if name__.is_some() {
                                return Err(serde::de::Error::duplicate_field("name"));
                            }
                            name__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Version => {
                            if version__.is_some() {
                                return Err(serde::de::Error::duplicate_field("version"));
                            }
                            version__ = Some(map_.next_value()?);
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
                Ok(InstrumentationScope {
                    name: name__.unwrap_or_default(),
                    version: version__.unwrap_or_default(),
                    attributes: attributes__.unwrap_or_default(),
                    dropped_attributes_count: dropped_attributes_count__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.common.v1.InstrumentationScope", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for KeyValue {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.key.is_empty() {
            len += 1;
        }
        if self.value.is_some() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.common.v1.KeyValue", len)?;
        if !self.key.is_empty() {
            struct_ser.serialize_field("key", &self.key)?;
        }
        if let Some(v) = self.value.as_ref() {
            struct_ser.serialize_field("value", v)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for KeyValue {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "key",
            "value",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Key,
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
                            "key" => Ok(GeneratedField::Key),
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
            type Value = KeyValue;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.common.v1.KeyValue")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<KeyValue, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut key__ = None;
                let mut value__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Key => {
                            if key__.is_some() {
                                return Err(serde::de::Error::duplicate_field("key"));
                            }
                            key__ = Some(map_.next_value()?);
                        }
                        GeneratedField::Value => {
                            if value__.is_some() {
                                return Err(serde::de::Error::duplicate_field("value"));
                            }
                            value__ = map_.next_value()?;
                        }
                    }
                }
                Ok(KeyValue {
                    key: key__.unwrap_or_default(),
                    value: value__,
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.common.v1.KeyValue", FIELDS, GeneratedVisitor)
    }
}
impl serde::Serialize for KeyValueList {
    #[allow(deprecated)]
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut len = 0;
        if !self.values.is_empty() {
            len += 1;
        }
        let mut struct_ser = serializer.serialize_struct("opentelemetry.proto.common.v1.KeyValueList", len)?;
        if !self.values.is_empty() {
            struct_ser.serialize_field("values", &self.values)?;
        }
        struct_ser.end()
    }
}
impl<'de> serde::Deserialize<'de> for KeyValueList {
    #[allow(deprecated)]
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        const FIELDS: &[&str] = &[
            "values",
        ];

        #[allow(clippy::enum_variant_names)]
        enum GeneratedField {
            Values,
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
                            "values" => Ok(GeneratedField::Values),
                            _ => Err(serde::de::Error::unknown_field(value, FIELDS)),
                        }
                    }
                }
                deserializer.deserialize_identifier(GeneratedVisitor)
            }
        }
        struct GeneratedVisitor;
        impl<'de> serde::de::Visitor<'de> for GeneratedVisitor {
            type Value = KeyValueList;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("struct opentelemetry.proto.common.v1.KeyValueList")
            }

            fn visit_map<V>(self, mut map_: V) -> std::result::Result<KeyValueList, V::Error>
                where
                    V: serde::de::MapAccess<'de>,
            {
                let mut values__ = None;
                while let Some(k) = map_.next_key()? {
                    match k {
                        GeneratedField::Values => {
                            if values__.is_some() {
                                return Err(serde::de::Error::duplicate_field("values"));
                            }
                            values__ = Some(map_.next_value()?);
                        }
                    }
                }
                Ok(KeyValueList {
                    values: values__.unwrap_or_default(),
                })
            }
        }
        deserializer.deserialize_struct("opentelemetry.proto.common.v1.KeyValueList", FIELDS, GeneratedVisitor)
    }
}
