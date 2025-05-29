use crate::proto::opentelemetry::common::v1::InstrumentationScope;
use crate::proto::opentelemetry::common::v1::{
    KeyValue,
    //KeyValueList,
};
use crate::proto::opentelemetry::logs::v1::{
    //
    LogRecord,
    LogsData,
    ResourceLogs,
    ScopeLogs,
};
use crate::proto::opentelemetry::resource::v1::Resource;

///////////////////////////////////////////////////////////////////////

pub trait LogsVisitor {
    type Return;

    fn visit_logs(self, v: impl LogsDataVisitor) -> Self::Return;
}

pub trait LogsDataVisitor {
    fn visit_logs_data(self, v: impl ResourceLogsVisitor);
}

pub trait ResourceLogsVisitor {
    fn visit_resource_logs(self);
}

///////////////////////////////////////////////////////////////////////

pub struct ItemCounter {
    count: usize,
}

impl ItemCounter {
    pub fn new() -> Self {
        Self { count: 0 }
    }
}

impl<'a> LogsVisitor for ItemCounter {
    type Return = usize;

    fn visit_logs(mut self, mut v: impl LogsDataVisitor) -> Self::Return {
        v.visit_logs_data(&mut self);
        self.count
    }
}

impl<'a> ResourceLogsVisitor for &mut ItemCounter {
    fn visit_resource_logs(self) {}
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

impl<'a> LogsDataVisitor for &mut LogsDataAdapter<'a> {
    fn visit_logs_data(self, mut v: impl ResourceLogsVisitor) {
        for rl in &self.data.resource_logs {
            //            let mut rla = ResourceLogsAdapter::new(rl);
            // let mut ra = ResourceAdapter::new(rl.resource.as_ref());
            // let mut sla = ScopeLogsAdapter::new(&rl.scope_logs);
            v.visit_resource_logs(); // &mut ra, &mut sla);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proto::opentelemetry::logs::v1::SeverityNumber;

    #[test]
    fn test_something() {
        let _x = [1, 2, 3];

        let mut ic = ItemCounter::new();

        let ld = LogsData::new(vec![
            ResourceLogs::build(Resource::new(vec![]))
                .scope_logs(vec![
                    ScopeLogs::build(InstrumentationScope::new("test"))
                        .log_records(vec![
                            LogRecord::new(1000000000u64, SeverityNumber::Info, "my_log"),
                            LogRecord::new(1000000001u64, SeverityNumber::Warn, "my_log warn"),
                        ])
                        .finish(),
                ])
                .finish(),
        ]);

        let ic = LogsDataAdapter::new(&ld).visit(ItemCounter::new());
        assert_eq!(2, ic.0);
    }
}
