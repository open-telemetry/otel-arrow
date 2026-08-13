// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::pdata::OtapPdata;
use otap_df_engine::error::Error;
use otap_df_pdata::proto::OtlpProtoMessage;

impl TryFrom<OtlpProtoMessage> for OtapPdata {
    type Error = Error;

    fn try_from(value: OtlpProtoMessage) -> Result<Self, Self::Error> {
        let payload = value
            .try_into()
            .map_err(|error: prost::EncodeError| Error::PDataError {
                reason: error.to_string(),
            })?;
        Ok(OtapPdata::new_todo_context(payload))
    }
}
