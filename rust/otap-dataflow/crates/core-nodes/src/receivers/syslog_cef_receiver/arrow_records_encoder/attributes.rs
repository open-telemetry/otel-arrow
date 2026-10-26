// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Syslog vocabulary for the shared opt-in known-key attribute builder.
//! Slots identify names; dictionary IDs and all Arrow storage stay in pdata.

use otel_arrow_dfe_pdata::encode::record::attributes::{
    KnownKey, KnownKeyArrayBuilder, StrKeysAttributesRecordBatchBuilder,
};

// Generate the enum, names, and test enumeration together so reordering or
// adding a variant cannot separate its vocabulary slot from its attribute name.
macro_rules! define_known_attribute_keys {
    ($($variant:ident => $name:literal),+ $(,)?) => {
        /// Slots for the fixed syslog and CEF-header vocabulary; extensions use strings.
        /// These are vocabulary positions, not batch-local Arrow dictionary IDs.
        #[derive(Clone, Copy)]
        pub(crate) enum KnownAttributeKey {
            $($variant),+
        }

        impl KnownAttributeKey {
            const COUNT: usize = [$($name),+].len();

            #[cfg(test)]
            const ALL: [Self; Self::COUNT] = [$(Self::$variant),+];

            pub(crate) const fn as_str(self) -> &'static str {
                match self {
                    $(Self::$variant => $name),+
                }
            }
        }

        const NAMES: [&str; KnownAttributeKey::COUNT] = [
            $(KnownAttributeKey::$variant.as_str()),+
        ];
    };
}

define_known_attribute_keys! {
    Facility => "syslog.facility",
    Severity => "syslog.severity",
    HostName => "syslog.host_name",
    Version => "syslog.version",
    AppName => "syslog.app_name",
    ProcessId => "syslog.process_id",
    ProcessIdStr => "syslog.process_id_str",
    MessageId => "syslog.msg_id",
    StructuredData => "syslog.structured_data",
    Message => "syslog.message",
    Tag => "syslog.tag",
    Content => "syslog.content",
    CefVersion => "cef.version",
    CefDeviceVendor => "cef.device_vendor",
    CefDeviceProduct => "cef.device_product",
    CefDeviceVersion => "cef.device_version",
    CefEventClassId => "cef.device_event_class_id",
    CefName => "cef.name",
    CefSeverity => "cef.severity",
    InputFormat => "input.format",
}

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

    /// Scenario: Every fixed vocabulary slot is mixed with dynamic names, repeated parents, and varied values.
    /// Guarantees: Complete attribute records retain exact fields, encoding metadata, row order, and values.
    #[test]
    fn syslog_attributes_match_general_builder() {
        for rows in [0, 1, 1000] {
            let mut actual = new_builder();
            let mut expected = StrKeysAttributesRecordBatchBuilder::<u16>::new();
            for parent in 0..rows {
                for key in KnownAttributeKey::ALL {
                    actual.append_known_key(key);
                    expected.append_key(key.as_str());
                    actual.any_values_builder.append_int(i64::from(parent));
                    expected.any_values_builder.append_int(i64::from(parent));
                }
                actual.append_key("dynamic");
                expected.append_key("dynamic");
                actual.any_values_builder.append_str(b"value");
                expected.any_values_builder.append_str(b"value");
                actual.append_known_key(KnownAttributeKey::InputFormat);
                expected.append_key(KnownAttributeKey::InputFormat.as_str());
                actual.any_values_builder.append_empty();
                expected.any_values_builder.append_empty();
                actual.append_parent_id_n(&parent, KnownAttributeKey::COUNT + 2);
                expected.append_parent_id_n(&parent, KnownAttributeKey::COUNT + 2);
            }
            assert_eq!(actual.finish().unwrap(), expected.finish().unwrap());
        }
    }
}
