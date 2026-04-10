// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Conversion helpers from internal admin/server types to public SDK models.

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
