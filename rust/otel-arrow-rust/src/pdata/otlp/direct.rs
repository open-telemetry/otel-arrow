///////////////////////////////////////////////////////////////////////
// The pattern below is to define two traits per OTLP data type, to
// demonstrate how the visitor pattern will work.  For productionizing
// this concept, we will extend the ./derive macro package.
//
// For each type there is a **Visitor** (an actor) and a
// **Visitable**. Using these traits allows an implementation to
// perform an in-order traversal of OTLP-like data structure.
//
// A Visitable impl is passed to the Visitor to enact the visitation.
// Visitable adapter structs are provided for presenting OTLP data
// types as Visitable impls. Visitable impls are immutable, passed by
// &self.
//
// Visitor impls are passed to the Visitable carrying logic as the
// traversal descends into a child. Visitors are provided for each
// field of the child. Visitors are mutable, passed by &mut self.
//
// In general, the Visitor calls the Visitable and the Visitable calls
// the child Visitors. At the top-level, users are directed to use for
// example, LogsVisitor which consumes the visitor and returns the
// associated **Return** type.
//
// As an example, to count log records we can use the visitor:
//
// pub fn LogRecordCount(ld: &LogsData) -> usize {
//     ItemCounter::new().visit_logs(&LogsDataAdapter::new(&ld))
// }
//
// Benchmark results for counting 10 resources * 10 scopes each * 10
// records each indicate a slowdown
//
// OTLP Logs counting/Visitor
//                         time:   [1.4456 ns 1.4878 ns 1.5354 ns]
// OTLP Logs counting/Manual
//                         time:   [930.08 ps 962.65 ps 996.07 ps]
//
// This is inconclusive. We expect some cost for the abstraction,
// and a more realistic application should be measured.

use crate::proto::opentelemetry::common::v1::InstrumentationScope as Scope;
use crate::proto::opentelemetry::logs::v1::*;
use crate::proto::opentelemetry::resource::v1::*;

/// LogsVisitor is the entry point for visiting OTLP logs data.
pub trait LogsVisitor {
    type Return;

    fn visit_logs(self, v: impl LogsDataVisitable) -> Self::Return;
}

// Visitor/Visitable for LogsData

pub trait LogsDataVisitor {
    fn visit_logs_data(&mut self, v: impl LogsDataVisitable);
}

pub trait LogsDataVisitable {
    fn visit_logs_data(&self, v: impl ResourceLogsVisitor);
}

// Visitor/Visitable for ResourceLogs

pub trait ResourceLogsVisitor {
    fn visit_resource_logs(&mut self, v: impl ResourceLogsVisitable);
}

pub trait ResourceLogsVisitable {
    fn visit_resource_logs(&self, rv: impl ResourceVisitor, slv: impl ScopeLogsVisitor);
}

// Visitor/Visitable for ScopeLogs

pub trait ScopeLogsVisitor {
    fn visit_scope_logs(&mut self, v: impl ScopeLogsVisitable);
}

pub trait ScopeLogsVisitable {
    fn visit_scope_logs(&self, sv: impl ScopeVisitor, lrv: impl LogRecordVisitor);
}

// Visitor/Visitable for Resoruce

pub trait ResourceVisitor {
    fn visit_resource(&mut self, v: impl ResourceVisitable);
}

pub trait ResourceVisitable {
    fn visit_resource(&self);
}

// Visitor/Visitable for Scope

pub trait ScopeVisitor {
    fn visit_scope(&mut self, v: impl ScopeVisitable);
}

pub trait ScopeVisitable {
    fn visit_scope(&self);
}

// Visitor/Visitable for LogRecord

pub trait LogRecordVisitor {
    fn visit_log_record(&mut self, v: impl LogRecordVisitable);
}

pub trait LogRecordVisitable {
    fn visit_log_record(&self);
}

//////////////////////////////////////////////////////////////////////

/// LogsData adapater for OTLP.
pub struct LogsDataAdapter<'a> {
    data: &'a LogsData,
}

impl<'a> LogsDataAdapter<'a> {
    pub fn new(data: &'a LogsData) -> Self {
        Self { data }
    }
}

impl<'a> LogsDataVisitable for &LogsDataAdapter<'a> {
    fn visit_logs_data(&self, mut v: impl ResourceLogsVisitor) {
        for rl in &self.data.resource_logs {
            v.visit_resource_logs(&ResourceLogsAdapter::new(rl));
        }
    }
}

//////////////////////////////////////////////////////////////////////

/// ResourceLogs adapater for OTLP.
pub struct ResourceLogsAdapter<'a> {
    data: &'a ResourceLogs,
}

impl<'a> ResourceLogsAdapter<'a> {
    pub fn new(data: &'a ResourceLogs) -> Self {
        Self { data }
    }
}

impl<'a> ResourceLogsVisitable for &ResourceLogsAdapter<'a> {
    fn visit_resource_logs(&self, mut rv: impl ResourceVisitor, mut slv: impl ScopeLogsVisitor) {
        self.data.resource.as_ref().map(|r| {
            rv.visit_resource(&ResourceAdapter::new(r));
        });

        for sl in &self.data.scope_logs {
            slv.visit_scope_logs(&ScopeLogsAdapter::new(sl));
        }
    }
}

//////////////////////////////////////////////////////////////////////

/// ScopeLogs adapater for OTLP.
pub struct ScopeLogsAdapter<'a> {
    data: &'a ScopeLogs,
}

impl<'a> ScopeLogsAdapter<'a> {
    pub fn new(data: &'a ScopeLogs) -> Self {
        Self { data }
    }
}

impl<'a> ScopeLogsVisitable for &ScopeLogsAdapter<'a> {
    fn visit_scope_logs(&self, mut sv: impl ScopeVisitor, mut lrv: impl LogRecordVisitor) {
        self.data.scope.as_ref().map(|s| {
            sv.visit_scope(&ScopeAdapter::new(s));
        });

        for lr in &self.data.log_records {
            lrv.visit_log_record(&LogRecordAdapter::new(lr));
        }
    }
}

//////////////////////////////////////////////////////////////////////

/// Resource adapater for OTLP.
pub struct ResourceAdapter<'a> {
    _data: &'a Resource,
}

impl<'a> ResourceAdapter<'a> {
    pub fn new(data: &'a Resource) -> Self {
        Self { _data: data }
    }
}

impl<'a> ResourceVisitable for &ResourceAdapter<'a> {
    // TODO: Add visitors for attributes, entity, schema_url, etc.
    fn visit_resource(&self) {}
}

//////////////////////////////////////////////////////////////////////

/// Scope adapater for OTLP.
pub struct ScopeAdapter<'a> {
    _data: &'a Scope,
}

impl<'a> ScopeAdapter<'a> {
    pub fn new(data: &'a Scope) -> Self {
        Self { _data: data }
    }
}

impl<'a> ScopeVisitable for &ScopeAdapter<'a> {
    // TODO: Add visitors for name, version, attributes, schema_url, etc.
    fn visit_scope(&self) {}
}

//////////////////////////////////////////////////////////////////////

/// LogRecord adapater for OTLP.
pub struct LogRecordAdapter<'a> {
    _data: &'a LogRecord,
}

impl<'a> LogRecordAdapter<'a> {
    pub fn new(data: &'a LogRecord) -> Self {
        Self { _data: data }
    }
}

impl<'a> LogRecordVisitable for &LogRecordAdapter<'a> {
    // TODO: Add visitors ...
    fn visit_log_record(&self) {}
}

///////////////////////////////////////////////////////////////////////

/// Noop implements a no-op for every visitor trait.
pub struct Noop {}

impl ResourceVisitor for Noop {
    fn visit_resource(&mut self, _: impl ResourceVisitable) {}
}

impl ScopeVisitor for Noop {
    fn visit_scope(&mut self, _: impl ScopeVisitable) {}
}

// TODO More Noop impls.

///////////////////////////////////////////////////////////////////////

/// ItemCounter implements counting log records.  TODO: The same could
/// count spans and metric data points.  Note: this sort of item
/// counting is a built-in feature of the Golang Pdata API.
pub struct ItemCounter {
    count: usize,
}

impl ItemCounter {
    pub fn new() -> Self {
        Self { count: 0 }
    }

    // TODO use the BorrowMut trait (for any reason)?
    fn borrow_mut<'a>(&'a mut self) -> &'a mut Self {
        self
    }
}

impl LogsVisitor for ItemCounter {
    type Return = usize;

    fn visit_logs(mut self, v: impl LogsDataVisitable) -> Self::Return {
        self.visit_logs_data(v);
        self.count
    }
}

impl LogsDataVisitor for ItemCounter {
    fn visit_logs_data(&mut self, v: impl LogsDataVisitable) {
        v.visit_logs_data(self.borrow_mut());
    }
}

impl ResourceLogsVisitor for &mut ItemCounter {
    fn visit_resource_logs(&mut self, v: impl ResourceLogsVisitable) {
        v.visit_resource_logs(Noop {}, self.borrow_mut());
    }
}

impl ScopeLogsVisitor for &mut ItemCounter {
    fn visit_scope_logs(&mut self, sv: impl ScopeLogsVisitable) {
        sv.visit_scope_logs(Noop {}, self.borrow_mut());
    }
}

impl LogRecordVisitor for &mut ItemCounter {
    fn visit_log_record(&mut self, _: impl LogRecordVisitable) {
        self.count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::opentelemetry::common::v1::*;

    #[test]
    fn test_logs_item_count() {
        let ld = LogsData::new(vec![
            ResourceLogs::build(Resource::new(vec![]))
                .scope_logs(vec![
                    ScopeLogs::build(InstrumentationScope::new("test0"))
                        .log_records(vec![
                            LogRecord::new(1000000000u64, SeverityNumber::Info, "my_log"),
                            LogRecord::new(1000000000u64, SeverityNumber::Info, "my_log"),
                            LogRecord::new(1000000000u64, SeverityNumber::Info, "my_log"),
                            LogRecord::new(1000000000u64, SeverityNumber::Info, "my_log"),
                            LogRecord::new(1000000000u64, SeverityNumber::Info, "my_log"),
                            LogRecord::new(1000000001u64, SeverityNumber::Warn, "my_log warn"),
                            LogRecord::new(1000000001u64, SeverityNumber::Warn, "my_log warn"),
                            LogRecord::new(1000000001u64, SeverityNumber::Warn, "my_log warn"),
                            LogRecord::new(1000000001u64, SeverityNumber::Warn, "my_log warn"),
                            LogRecord::new(1000000001u64, SeverityNumber::Warn, "my_log warn"),
                        ])
                        .finish(),
                    ScopeLogs::build(InstrumentationScope::new("test1"))
                        .log_records(vec![
                            LogRecord::new(1000000000u64, SeverityNumber::Info, "my_log"),
                            LogRecord::new(1000000000u64, SeverityNumber::Info, "my_log"),
                            LogRecord::new(1000000000u64, SeverityNumber::Info, "my_log"),
                            LogRecord::new(1000000000u64, SeverityNumber::Info, "my_log"),
                            LogRecord::new(1000000000u64, SeverityNumber::Info, "my_log"),
                            LogRecord::new(1000000001u64, SeverityNumber::Warn, "my_log warn"),
                            LogRecord::new(1000000001u64, SeverityNumber::Warn, "my_log warn"),
                            LogRecord::new(1000000001u64, SeverityNumber::Warn, "my_log warn"),
                            LogRecord::new(1000000001u64, SeverityNumber::Warn, "my_log warn"),
                            LogRecord::new(1000000001u64, SeverityNumber::Warn, "my_log warn"),
                        ])
                        .finish(),
                ])
                .finish(),
        ]);

        let ic = ItemCounter::new().visit_logs(&LogsDataAdapter::new(&ld));
        assert_eq!(20, ic);
    }
}
