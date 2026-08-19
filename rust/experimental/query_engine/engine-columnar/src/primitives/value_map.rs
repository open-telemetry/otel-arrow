// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::rc::Rc;

use ahash::AHashMap;
use data_engine_expressions::*;

use crate::*;

#[derive(Debug, Clone)]
pub enum MapValueOrRef<'a> {
    Ref(&'a (dyn MapValue + 'a)),
    Owned(Rc<OwnedMapValue<'a>>),
}

impl<'a> MapValueOrRef<'a> {
    pub fn as_map_value(&self) -> &'_ (dyn MapValue + 'a) {
        match self {
            MapValueOrRef::Ref(m) => *m,
            MapValueOrRef::Owned(m) => m.as_ref(),
        }
    }
}

impl Hash for MapValueOrRef<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        [8].hash(state);
        let mut acc = 0u64;
        match self {
            MapValueOrRef::Ref(m) => {
                m.len().hash(state);
                m.get_items(&mut |k, v| {
                    let mut h = ahash::AHasher::default();
                    k.hash(&mut h);
                    Into::<ValueOrRef>::into(v).hash(&mut h);
                    acc = acc.wrapping_add(h.finish()); // order-independent
                    true
                });
            }
            MapValueOrRef::Owned(m) => {
                m.len().hash(state);
                for (k, v) in &m.values {
                    let mut h = ahash::AHasher::default();
                    k.hash(&mut h);
                    v.hash(&mut h);
                    acc = acc.wrapping_add(h.finish()); // order-independent
                }
            }
        }
        acc.hash(state);
    }
}

#[derive(Debug, Clone)]
pub struct OwnedMapValue<'a> {
    values: AHashMap<Box<str>, ValueOrRef<'a>>,
}

impl<'a> OwnedMapValue<'a> {
    pub fn new() -> OwnedMapValue<'a> {
        Self {
            values: AHashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> OwnedMapValue<'a> {
        Self {
            values: AHashMap::with_capacity(capacity),
        }
    }

    pub fn get_values(&self) -> &AHashMap<Box<str>, ValueOrRef<'a>> {
        &self.values
    }

    pub fn get_values_mut(&mut self) -> &mut AHashMap<Box<str>, ValueOrRef<'a>> {
        &mut self.values
    }
}

impl<'a, const N: usize> From<[(Box<str>, ValueOrRef<'a>); N]> for MapValueOrRef<'a> {
    fn from(arr: [(Box<str>, ValueOrRef<'a>); N]) -> Self {
        MapValueOrRef::Owned(
            OwnedMapValue {
                values: AHashMap::<Box<str>, ValueOrRef<'a>>::from_iter(arr),
            }
            .into(),
        )
    }
}

impl<'a> MapValue for OwnedMapValue<'a> {
    fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    fn len(&self) -> usize {
        self.values.len()
    }

    fn contains_key(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    fn get(&self, key: &str) -> Option<&(dyn AsValue + 'a)> {
        self.values.get(key).map(|v| v as &dyn AsValue)
    }

    fn get_static(&self, _key: &str) -> Result<Option<&(dyn AsStaticValue + 'static)>, String> {
        unreachable!("should never be called by columnar engine")
    }

    fn get_items<'b>(&'b self, item_callback: &mut MapValueIteratorCallback<'b, '_>) -> bool {
        for (key, value) in self.values.iter() {
            if !(item_callback)(key, value.to_value()) {
                return false;
            }
        }

        true
    }
}

impl<'a> Default for OwnedMapValue<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> From<&'a (dyn MapValue + 'a)> for OwnedMapValue<'a> {
    fn from(value: &'a (dyn MapValue + 'a)) -> Self {
        let mut map = OwnedMapValue::with_capacity(value.len());

        let values = map.get_values_mut();

        value.get_items(&mut |key, value| {
            values.insert(key.into(), value.into());
            true
        });

        map
    }
}

impl<'a> From<MapValueOrRef<'a>> for OwnedMapValue<'a> {
    fn from(value: MapValueOrRef<'a>) -> Self {
        match value {
            MapValueOrRef::Owned(map) => Rc::unwrap_or_clone(map),
            MapValueOrRef::Ref(map) => (*map).into(),
        }
    }
}
