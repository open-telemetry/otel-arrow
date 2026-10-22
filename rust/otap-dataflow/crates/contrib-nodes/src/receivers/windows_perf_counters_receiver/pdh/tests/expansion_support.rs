// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::super::native::NativePdh;
use super::*;

pub(in super::super) fn parse_instance(path: &str) -> Result<InstanceIdentity, Error> {
    NativePdh::parse_instance_path(path)
}
