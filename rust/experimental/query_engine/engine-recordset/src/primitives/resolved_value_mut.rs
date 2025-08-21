// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::cell::RefMut;

use crate::*;

pub enum ResolvedValueMut<'a, 'b> {
    Map(RefMut<'a, dyn MapValueMut + 'static>),
    MapKey {
        map: RefMut<'a, dyn MapValueMut + 'static>,
        key: ResolvedStringValue<'b>,
    },
    ArrayIndex {
        array: RefMut<'a, dyn ArrayValueMut + 'static>,
        index: usize,
    },
}
