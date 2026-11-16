// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Encoder cases sharing the native-log fixture and owned-output timing path.

use otel_arrow_dfe_config::codecs::{CodecBinding, CodecConfiguration, CodecsConfig};
use otel_arrow_dfe_pdata::OtapArrowRecords;
use otel_arrow_dfe_pdata_codec::PdataEncoding;

pub struct EncoderCase {
    pub name: &'static str,
    pub encoding: PdataEncoding,
    pub configuration: CodecsConfig,
    pub fixture: fn(usize) -> OtapArrowRecords,
}

pub fn encoders() -> [EncoderCase; 7] {
    [
        EncoderCase {
            name: "otlp",
            encoding: PdataEncoding::OTLP,
            configuration: CodecsConfig::default(),
            fixture: super::fixtures::syslog_logs,
        },
        syslog("rfc3164"),
        syslog("rfc5424"),
        syslog("cef"),
        syslog("cef_rfc3164"),
        syslog("cef_rfc5424"),
        syslog("all"),
    ]
}

fn syslog(profile: &'static str) -> EncoderCase {
    EncoderCase {
        name: profile,
        encoding: PdataEncoding::new("syslog-batch"),
        configuration: CodecsConfig {
            bindings: [(
                "syslog-batch".into(),
                CodecBinding::Inline(CodecConfiguration {
                    options: [("profile".into(), profile.into())].into(),
                    ..Default::default()
                }),
            )]
            .into(),
            ..Default::default()
        },
        fixture: super::fixtures::syslog_logs,
    }
}
