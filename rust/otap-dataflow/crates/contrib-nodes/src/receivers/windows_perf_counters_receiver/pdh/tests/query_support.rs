// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

impl<A: PdhApi> Query<A> {
    pub(in super::super) fn set_max_native_counters(
        &mut self,
        max_native_counters: usize,
    ) -> Result<(), Error> {
        if self.exact_counter_reservations > max_native_counters {
            return Err(Error::InvalidSample(
                "query native-counter limit is below its exact-counter reservation",
            ));
        }
        self.max_native_counters = max_native_counters;
        Ok(())
    }
}
