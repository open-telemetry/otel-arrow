// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Form OTLP Resource encodings from the configuration struct

use bytes::Bytes;
use otap_df_config::pipeline::service::telemetry::AttributeValue;
use otap_df_pdata::otlp::ProtoBuffer;
use otap_df_pdata::proto::consts::field_num::common::{
    ANY_VALUE_BOOL_VALUE, ANY_VALUE_DOUBLE_VALUE, ANY_VALUE_INT_VALUE, ANY_VALUE_STRING_VALUE,
    KEY_VALUE_KEY, KEY_VALUE_VALUE,
};
use otap_df_pdata::proto::consts::field_num::logs::RESOURCE_LOGS_RESOURCE;
use otap_df_pdata::proto::consts::field_num::resource::RESOURCE_ATTRIBUTES;
use otap_df_pdata::proto::consts::wire_types;
use otap_df_pdata::proto_encode_len_delimited_unknown_size;
use std::collections::HashMap;

/// Encode OTLP bytes of the ResourceLogs.resource field, the whole
/// tag-and-bytes representation for a single copy.
#[must_use]
pub fn encode_resource_bytes(attrs: &HashMap<String, AttributeValue>) -> Bytes {
    if attrs.is_empty() {
        return Bytes::new();
    }

    let mut buf = ProtoBuffer::with_capacity(attrs.len() * 64);

    // Encode: field 1 (RESOURCE_LOGS_RESOURCE) -> Resource message
    proto_encode_len_delimited_unknown_size!(
        RESOURCE_LOGS_RESOURCE,
        {
            // Resource { attributes: [ KeyValue, ... ] }
            for (key, value) in attrs {
                encode_resource_attribute(&mut buf, key, value);
            }
        },
        &mut buf
    );

    buf.into_bytes()
}

/// Encode a single resource attribute as a KeyValue message.
fn encode_resource_attribute(buf: &mut ProtoBuffer, key: &str, value: &AttributeValue) {
    proto_encode_len_delimited_unknown_size!(
        RESOURCE_ATTRIBUTES,
        {
            buf.encode_string(KEY_VALUE_KEY, key);
            proto_encode_len_delimited_unknown_size!(
                KEY_VALUE_VALUE,
                {
                    match value {
                        AttributeValue::String(s) => {
                            buf.encode_string(ANY_VALUE_STRING_VALUE, s);
                        }
                        AttributeValue::Bool(b) => {
                            buf.encode_field_tag(ANY_VALUE_BOOL_VALUE, wire_types::VARINT);
                            buf.encode_varint(u64::from(*b));
                        }
                        AttributeValue::I64(i) => {
                            buf.encode_field_tag(ANY_VALUE_INT_VALUE, wire_types::VARINT);
                            buf.encode_varint(*i as u64);
                        }
                        AttributeValue::F64(f) => {
                            buf.encode_field_tag(ANY_VALUE_DOUBLE_VALUE, wire_types::FIXED64);
                            buf.extend_from_slice(&f.to_le_bytes());
                        }
                        AttributeValue::Array(_) => {
                            crate::raw_error!("Arrays are not supported in resource attributes");
                        }
                    }
                },
                buf
            );
        },
        buf
    );
}
