use std::cell::RefMut;

use crate::*;

#[allow(dead_code)]
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
