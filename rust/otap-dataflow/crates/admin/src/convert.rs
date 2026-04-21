// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Conversion helpers from internal admin/server types to public SDK models.

use otap_df_admin_types::telemetry as api;
use otap_df_telemetry::attributes::AttributeValue;
use serde::Serialize;
use serde::de::DeserializeOwned;

pub(crate) fn json_shape<T, U>(value: &T) -> U
where
    T: Serialize,
    U: DeserializeOwned,
{
    serde_json::from_value(
        serde_json::to_value(value).expect("internal admin type should serialize"),
    )
    .expect("public admin model should deserialize from the current wire shape")
}

/// Convert an engine `AttributeValue` to the public admin API representation.
pub(crate) fn convert_attribute_value(value: &AttributeValue) -> api::AttributeValue {
    match value {
        AttributeValue::String(s) => api::AttributeValue::String(s.clone()),
        AttributeValue::Int(v) => api::AttributeValue::Int(*v),
        AttributeValue::UInt(v) => api::AttributeValue::UInt(*v),
        AttributeValue::Double(v) => api::AttributeValue::Double(*v),
        AttributeValue::Boolean(v) => api::AttributeValue::Boolean(*v),
        AttributeValue::Map(m) => api::AttributeValue::Map(
            m.iter()
                .map(|(k, v)| (k.clone(), convert_attribute_value(v)))
                .collect(),
        ),
    }
}
