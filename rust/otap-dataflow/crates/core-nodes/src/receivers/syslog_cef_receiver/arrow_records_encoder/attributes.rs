// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Syslog vocabulary for the shared opt-in known-key attribute builder.
//! Slots identify names; dictionary IDs and all Arrow storage stay in pdata.

use otel_arrow_dfe_pdata::encode::record::attributes::{
    KnownKey, KnownKeyArrayBuilder, StrKeysAttributesRecordBatchBuilder,
};

/// Slots for the fixed syslog and CEF-header vocabulary; extensions use strings.
/// Discriminants index the ID cache, not the Arrow dictionary: dictionary IDs
/// follow the order in which keys first occur within each batch.
#[derive(Clone, Copy)]
pub(crate) enum KnownAttributeKey {
    Facility,
    Severity,
    HostName,
    Version,
    AppName,
    ProcessId,
    ProcessIdStr,
    MessageId,
    StructuredData,
    Message,
    Tag,
    Content,
    CefVersion,
    CefDeviceVendor,
    CefDeviceProduct,
    CefDeviceVersion,
    CefEventClassId,
    CefName,
    CefSeverity,
    InputFormat,
}

impl KnownAttributeKey {
    const COUNT: usize = Self::InputFormat as usize + 1;

    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Facility => "syslog.facility",
            Self::Severity => "syslog.severity",
            Self::HostName => "syslog.host_name",
            Self::Version => "syslog.version",
            Self::AppName => "syslog.app_name",
            Self::ProcessId => "syslog.process_id",
            Self::ProcessIdStr => "syslog.process_id_str",
            Self::MessageId => "syslog.msg_id",
            Self::StructuredData => "syslog.structured_data",
            Self::Message => "syslog.message",
            Self::Tag => "syslog.tag",
            Self::Content => "syslog.content",
            Self::CefVersion => "cef.version",
            Self::CefDeviceVendor => "cef.device_vendor",
            Self::CefDeviceProduct => "cef.device_product",
            Self::CefDeviceVersion => "cef.device_version",
            Self::CefEventClassId => "cef.device_event_class_id",
            Self::CefName => "cef.name",
            Self::CefSeverity => "cef.severity",
            Self::InputFormat => "input.format",
        }
    }
}

const NAMES: [&str; KnownAttributeKey::COUNT] = [
    KnownAttributeKey::Facility.as_str(),
    KnownAttributeKey::Severity.as_str(),
    KnownAttributeKey::HostName.as_str(),
    KnownAttributeKey::Version.as_str(),
    KnownAttributeKey::AppName.as_str(),
    KnownAttributeKey::ProcessId.as_str(),
    KnownAttributeKey::ProcessIdStr.as_str(),
    KnownAttributeKey::MessageId.as_str(),
    KnownAttributeKey::StructuredData.as_str(),
    KnownAttributeKey::Message.as_str(),
    KnownAttributeKey::Tag.as_str(),
    KnownAttributeKey::Content.as_str(),
    KnownAttributeKey::CefVersion.as_str(),
    KnownAttributeKey::CefDeviceVendor.as_str(),
    KnownAttributeKey::CefDeviceProduct.as_str(),
    KnownAttributeKey::CefDeviceVersion.as_str(),
    KnownAttributeKey::CefEventClassId.as_str(),
    KnownAttributeKey::CefName.as_str(),
    KnownAttributeKey::CefSeverity.as_str(),
    KnownAttributeKey::InputFormat.as_str(),
];

const SLOTS: [KnownKey<{ KnownAttributeKey::COUNT }>; KnownAttributeKey::COUNT] = KnownKey::slots();

impl From<KnownAttributeKey> for KnownKey<{ KnownAttributeKey::COUNT }> {
    fn from(key: KnownAttributeKey) -> Self {
        // Every enum discriminant indexes the corresponding vocabulary entry.
        SLOTS[key as usize]
    }
}

pub(crate) type SyslogAttributesBuilder =
    StrKeysAttributesRecordBatchBuilder<u16, KnownKeyArrayBuilder<{ KnownAttributeKey::COUNT }>>;

pub(super) fn new_builder() -> SyslogAttributesBuilder {
    SyslogAttributesBuilder::new_with_known_keys(&NAMES)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: Mixed fixed and dynamic attributes use repeated parents and string, integer, and empty values.
    /// Guarantees: Complete attribute records retain exact fields, encoding metadata, row order, and values.
    #[test]
    fn syslog_attributes_match_general_builder() {
        for rows in [0, 1, 1000] {
            let mut actual = new_builder();
            let mut expected = StrKeysAttributesRecordBatchBuilder::<u16>::new();
            for parent in 0..rows {
                actual.append_known_key(KnownAttributeKey::Facility);
                expected.append_key(KnownAttributeKey::Facility.as_str());
                actual.any_values_builder.append_int(i64::from(parent));
                expected.any_values_builder.append_int(i64::from(parent));
                actual.append_key("dynamic");
                expected.append_key("dynamic");
                actual.any_values_builder.append_str(b"value");
                expected.any_values_builder.append_str(b"value");
                actual.append_known_key(KnownAttributeKey::InputFormat);
                expected.append_key(KnownAttributeKey::InputFormat.as_str());
                actual.any_values_builder.append_empty();
                expected.any_values_builder.append_empty();
                actual.append_parent_id_n(&parent, 3);
                expected.append_parent_id_n(&parent, 3);
            }
            assert_eq!(actual.finish().unwrap(), expected.finish().unwrap());
        }
    }
}
