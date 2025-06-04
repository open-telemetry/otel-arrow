// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This module contains traits and utilities for OTLP (OpenTelemetry Protocol) message types.

// Re-export derive macros (required for generated code)
pub use otlp_derive::Message;
pub use otlp_derive::qualified;

use crate::proto::opentelemetry::logs::v1::LogRecordVisitable;
use crate::proto::opentelemetry::logs::v1::LogRecordVisitor;
use crate::proto::opentelemetry::logs::v1::LogsDataVisitable;
use crate::proto::opentelemetry::logs::v1::LogsDataVisitor;
use crate::proto::opentelemetry::logs::v1::ResourceLogsVisitable;
use crate::proto::opentelemetry::logs::v1::ResourceLogsVisitor;
use crate::proto::opentelemetry::logs::v1::ScopeLogsVisitable;
use crate::proto::opentelemetry::logs::v1::ScopeLogsVisitor;

/// LogsVisitor is the entry point for visiting OTLP logs data.
pub trait LogsVisitor<Argument> {
    /// The return type of the visitor
    type Return;

    /// Visit logs data and return the computed result
    fn visit_logs(self, v: impl LogsDataVisitable<Argument>) -> Self::Return;
}

/// ItemCounter implements counting log records. This sort of item
/// counting is a built-in feature of the Golang Pdata API.
pub struct ItemCounter {
    count: usize,
}

impl ItemCounter {
    /// Create a new ItemCounter starting at 0
    pub fn new() -> Self {
        Self { count: 0 }
    }

    #[allow(dead_code)] // Will be used when full adapter pattern is implemented
    fn borrow_mut<'a>(&'a mut self) -> &'a mut Self {
        self
    }
}

impl LogsVisitor<()> for ItemCounter {
    /// The return type of the visitor
    type Return = usize;

    /// Visit logs data and return the computed result
    fn visit_logs(mut self, v: impl LogsDataVisitable<()>) -> Self::Return {
        v.accept_logs_data((), &mut self);
        self.count
    }
}

impl<Argument> LogsDataVisitor<Argument> for ItemCounter {
    fn visit_logs_data(&mut self, arg: Argument, v: impl LogsDataVisitable<Argument>) -> Argument {
        v.accept_logs_data(arg, self.borrow_mut())
    }
}

impl<Argument> ResourceLogsVisitor<Argument> for &mut ItemCounter {
    fn visit_resource_logs(
        &mut self,
        arg: Argument,
        v: impl ResourceLogsVisitable<Argument>,
    ) -> Argument {
        v.accept_resource_logs(
            arg,
            super::NoopVisitor {},
            self.borrow_mut(),
            super::NoopVisitor {},
        )
    }
}

impl<Argument> ScopeLogsVisitor<Argument> for &mut ItemCounter {
    fn visit_scope_logs(
        &mut self,
        arg: Argument,
        sv: impl ScopeLogsVisitable<Argument>,
    ) -> Argument {
        sv.accept_scope_logs(
            arg,
            super::NoopVisitor {},
            self.borrow_mut(),
            super::NoopVisitor {},
        )
    }
}

impl<Argument> LogRecordVisitor<Argument> for &mut ItemCounter {
    fn visit_log_record(
        &mut self,
        arg: Argument,
        _: impl LogRecordVisitable<Argument>,
    ) -> Argument {
        self.count += 1;
        arg
    }
}

#[cfg(test)]
mod tests;
