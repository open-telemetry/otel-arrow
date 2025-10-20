// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;

use async_trait::async_trait;
use otel_arrow_rust::otap::OtapArrowRecords;

use crate::error::Result;

#[async_trait]
pub trait OutPort {
    async fn receive(&mut self, batch: OtapArrowRecords) -> Result<()>;
}

pub trait OutPortProvider<T: OutPort> {
    fn get_by_name(&mut self, name: &str) -> Option<&mut T>;
}

// simple collection based implementations for usage in development, examples, testing, etc.

pub struct MapOutPortProvider<T: OutPort> {
    inner: BTreeMap<String, T>,
}

impl<T> MapOutPortProvider<T>
where
    T: OutPort + Default,
{
    pub fn new_with_names<S, I>(names: I) -> Self
    where
        S: ToString,
        I: IntoIterator<Item = S>,
    {
        let mut inner = BTreeMap::new();
        names
            .into_iter()
            .for_each(|name| _ = inner.insert(name.to_string(), T::default()));

        Self { inner }
    }
}

impl<T> OutPortProvider<T> for MapOutPortProvider<T>
where
    T: OutPort,
{
    fn get_by_name(&mut self, name: &str) -> Option<&mut T> {
        self.inner.get_mut(name)
    }
}

#[derive(Default)]
struct VecReceiverOutPort {
    inner: Vec<OtapArrowRecords>,
}

#[async_trait]
impl OutPort for VecReceiverOutPort {
    async fn receive(&mut self, batch: OtapArrowRecords) -> Result<()> {
        self.inner.push(batch);
        Ok(())
    }
}
