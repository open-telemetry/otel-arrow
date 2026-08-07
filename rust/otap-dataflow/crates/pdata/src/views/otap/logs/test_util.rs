// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arrow::datatypes::{DataType, Field};

use crate::schema::{FieldExt, consts};

pub(super) fn plain_field(name: &str, data_type: DataType, nullable: bool) -> Field {
    Field::new(name, data_type, nullable).with_plain_encoding()
}

pub(super) fn plain_id_field(nullable: bool) -> Field {
    plain_field(consts::ID, DataType::UInt16, nullable)
}

pub(super) fn plain_parent_id_field(nullable: bool) -> Field {
    plain_field(consts::PARENT_ID, DataType::UInt16, nullable)
}

pub(super) fn encoded_field(
    name: &str,
    data_type: DataType,
    nullable: bool,
    encoding: &str,
) -> Field {
    Field::new(name, data_type, nullable).with_encoding(encoding)
}
