use crate::proto::opentelemetry::common::v1::InstrumentationScope as Scope;
//use crate::proto::opentelemetry::common::v1::*;
// use crate::proto::opentelemetry::common::v1::{
//     KeyValue,
//     //KeyValueList,
// };
use crate::proto::opentelemetry::logs::v1::*;
// LogRecord,
// LogsData,
// ResourceLogs,
// Resource,
// ScopeLogs,
//};
use crate::proto::opentelemetry::resource::v1::*;

///////////////////////////////////////////////////////////////////////

pub trait LogsVisitor {
    type Return;

    fn visit_logs_data(self, v: impl LogsDataVisitable) -> Self::Return;
}

pub trait LogsDataVisitable {
    fn visit_logs_data(&self, v: impl ResourceLogsVisitor);
}

//

pub trait ResourceLogsVisitor {
    fn visit_resource_logs(&mut self, v: impl ResourceLogsVisitable);
}

pub trait ResourceLogsVisitable {
    fn visit_resource_logs(&self, rv: impl ResourceVisitor, slv: impl ScopeLogsVisitor);
}

//

pub trait ScopeLogsVisitor {
    fn visit_scope_logs(&mut self, v: impl ScopeLogsVisitable);
}

pub trait ScopeLogsVisitable {
    fn visit_scope_logs(&self, sv: impl ScopeVisitor, lrv: impl LogRecordVisitor);
}

//

pub trait ResourceVisitor {
    fn visit_resource(&mut self, v: impl ResourceVisitable);
}

pub trait ResourceVisitable {
    fn visit_resource(&self);
}

//

pub trait ScopeVisitor {
    fn visit_scope(&mut self, v: impl ScopeVisitable);
}

pub trait ScopeVisitable {
    fn visit_scope(&self);
}

//

pub trait LogRecordVisitor {
    fn visit_log_record(&mut self, v: impl LogRecordVisitable);
}

pub trait LogRecordVisitable {
    fn visit_log_record(&self);
}

//////////////////////////////////////////////////////////////////////

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

pub struct ResourceAdapter<'a> {
    _data: &'a Resource,
}

impl<'a> ResourceAdapter<'a> {
    pub fn new(data: &'a Resource) -> Self {
        Self { _data: data }
    }
}

impl<'a> ResourceVisitable for &ResourceAdapter<'a> {
    fn visit_resource(&self) {}
}

//////////////////////////////////////////////////////////////////////

pub struct ScopeAdapter<'a> {
    _data: &'a Scope,
}

impl<'a> ScopeAdapter<'a> {
    pub fn new(data: &'a Scope) -> Self {
        Self { _data: data }
    }
}

impl<'a> ScopeVisitable for &ScopeAdapter<'a> {
    fn visit_scope(&self) {}
}

//////////////////////////////////////////////////////////////////////

pub struct LogRecordAdapter<'a> {
    _data: &'a LogRecord,
}

impl<'a> LogRecordAdapter<'a> {
    pub fn new(data: &'a LogRecord) -> Self {
        Self { _data: data }
    }
}

impl<'a> LogRecordVisitable for &LogRecordAdapter<'a> {
    fn visit_log_record(&self) {}
}

///////////////////////////////////////////////////////////////////////

pub struct Noop {}

impl<'a> ResourceVisitor for Noop {
    fn visit_resource(&mut self, _: impl ResourceVisitable) {}
}

impl<'a> ScopeVisitor for Noop {
    fn visit_scope(&mut self, _: impl ScopeVisitable) {}
}

///////////////////////////////////////////////////////////////////////

pub struct ItemCounter {
    count: usize,
}

impl ItemCounter {
    pub fn new() -> Self {
        Self { count: 0 }
    }

    fn borrow_mut<'a>(&'a mut self) -> &'a mut Self {
        self
    }
}

impl LogsVisitor for ItemCounter {
    type Return = usize;

    fn visit_logs_data(mut self, v: impl LogsDataVisitable) -> Self::Return {
        v.visit_logs_data(&mut self);
        self.count
    }
}

impl<'a> ResourceLogsVisitor for &mut ItemCounter {
    fn visit_resource_logs(&mut self, v: impl ResourceLogsVisitable) {
        v.visit_resource_logs(Noop {}, self.borrow_mut());
    }
}

impl<'a> ScopeLogsVisitor for &mut ItemCounter {
    fn visit_scope_logs(&mut self, sv: impl ScopeLogsVisitable) {
        sv.visit_scope_logs(Noop {}, self.borrow_mut());
    }
}

impl<'a> LogRecordVisitor for &mut ItemCounter {
    fn visit_log_record(&mut self, _: impl LogRecordVisitable) {
        self.count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::opentelemetry::common::v1::*;
    //use crate::proto::opentelemetry::logs::v1::*;
    //use crate::proto::opentelemetry::resource::v1::*;

    #[test]
    fn test_something() {
        let _x = [1, 2, 3];

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

        let ic = ItemCounter::new().visit_logs_data(&LogsDataAdapter::new(&ld));
        assert_eq!(20, ic);
    }
}
