use crate::proto::opentelemetry::common::v1::InstrumentationScope;
use crate::proto::opentelemetry::logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs};
use crate::proto::opentelemetry::resource::v1::Resource;
// use crate::proto::opentelemetry::common::v1::KeyValue;
// use crate::proto::opentelemetry::common::v1::KeyValueList;

///////////////////////////////////////////////////////////////////////

pub struct ItemCounter(usize);

impl ItemCounter {
    pub fn new() -> Self {
        Self(0)
    }
}

///////////////////////////////////////////////////////////////////////

pub struct LogsDataAdapter<'a> {
    data: Option<&'a LogsData>,
}

impl<'a> LogsDataAdapter<'a> {
    pub fn new(data: Option<&'a LogsData>) -> Self {
        Self { data }
    }
}

impl<'a> LogsDataVisitor for LogsDataAdapter<'a> {
    fn visit_logs_data(&mut self, mut v: impl ResourceLogsVisitor) {
        self.data.map(|ld| {
            for rl in &ld.resource_logs {
                let mut ra = ResourceAdapter::new(rl.resource.as_ref());
                let mut sla = ScopeLogsAdapter::new(&rl.scope_logs);
                v.visit_resource_logs(&mut ra, &mut sla)
            }
        });
    }
}

///////////////////////////////////////////////////////////////////////

pub struct ResourceLogsAdapter<'a> {
    data: Option<&'a ResourceLogs>,
}

impl<'a> ResourceLogsAdapter<'a> {
    pub fn new(data: Option<&'a ResourceLogs>) -> Self {
        Self { data }
    }
}

impl<'a> ResourceLogsVisitor for ResourceLogsAdapter<'a> {
    fn visit_resource_logs(&mut self, rv: impl ResourceVisitor, slv: impl ScopeLogsVisitor) {
        self.data.map(|rl| {
            rl.resource.as_ref().map(|r| {
                let mut aa = AttributesAdapter::new(&r.attributes);
                rv.visit_resource(&mut aa);
            });
            for sl in &self.data.scope_logs {
                let mut sa = ScopeAdapter::new(&sl.scope);
                let mut rla = LogRecordsAdapter::new(&sl.log_records);
                slv.visit_scope_logs(&mut sa, &mut rla)
            }
        });
    }
}

///////////////////////////////////////////////////////////////////////

pub struct ResourceAdapter<'a> {
    data: Option<&'a Resource>,
}

impl<'a> ResourceAdapter<'a> {
    pub fn new(data: Option<&'a Resource>) -> Self {
        Self { data }
    }
}

impl<'a> ResourceVisitor for &mut ResourceAdapter<'a> {
    fn visit_resource(&mut self, a: impl AttributesVisitor) {
        for sl in &self.data.attributes {
            a.visit_attribute()
        }
    }
}

///////////////////////////////////////////////////////////////////////

pub struct ScopeLogsAdapter<'a> {
    data: &'a Vec<ScopeLogs>,
}

impl<'a> ScopeLogsAdapter<'a> {
    pub fn new(data: &'a Vec<ScopeLogs>) -> Self {
        Self { data }
    }
}

impl<'a> ScopeLogsVisitor for &mut ScopeLogsAdapter<'a> {
    fn visit_scope_logs(&mut self, sv: impl ScopeVisitor, lrv: impl LogRecordVisitor) {
        self.data.scope.as_ref().map(|sl| {
            sl.scope
                .as_ref()
                .map(|s| sv.visit_scope(&ScopeAdapater::new(s)))
        });
        for lr in self.data.log_records {
            lrv.visit_log_record(&LogRecordAdapter::new(lr))
        }
    }
}

///////////////////////////////////////////////////////////////////////

pub struct ScopeAdapter<'a> {
    data: Option<&'a InstrumentationScope>,
}

impl<'a> ScopeAdapter<'a> {
    pub fn new(data: Option<&'a InstrumentationScope>) -> Self {
        Self { data }
    }
}

impl<'a> ScopeVisitor for ScopeAdapter<'a> {
    fn visit_scope(
        &mut self,
        nv: impl StringVisitor,
        vv: impl StringVisitor,
        av: impl AttributesVisitor,
    ) {
        self.data.name.map(|name| nv.visit_string(name));
        self.data.version.map(|ver| vv.visit_string(ver));

        // if let Some(scope) = sl.scope.as_ref() {
        //     self.visit_scope(scope);
        // }
        for a in &self.data.attributes {
            self.visit_attributes(AttributesAdapter(a));
        }
    }
}

///////////////////////////////////////////////////////////////////////

pub trait LogsVisitor {
    type Return;

    fn visit_logs(&mut self, v: impl LogsDataVisitor) -> Self::Return;
}

pub trait LogsDataVisitor {
    fn visit_logs_data(&mut self, v: impl ResourceLogsVisitor);
}

pub trait ResourceLogsVisitor {
    fn visit_resource_logs(&mut self, r: impl ResourceVisitor, s: impl ScopeLogsVisitor);
}

pub trait ResourceVisitor {
    fn visit_resource(&mut self, r: impl AttributesVisitor);
}

pub trait ScopeLogsVisitor {
    fn visit_scope_logs(&mut self, r: impl ScopeVisitor, s: impl LogRecordVisitor);
}

pub trait AttributesVisitor {
    fn visit_attribute(&mut self);
}

pub trait ScopeVisitor {
    fn visit_scope(
        &mut self,
        nv: impl StringVisitor,
        vv: impl StringVisitor,
        av: impl AttributesVisitor,
    );
}

pub trait LogRecordVisitor {}

pub trait StringVisitor {}

// , k: impl KeyVisitor, s: impl ValueVisitor
// pub trait KeyVisitor {
//     fn visit_key(&mut self);
// }

// pub trait ValueVisitor {
//     fn visit_int(&mut self);
// }

//impl<'a, T> LogsAdapter<'a, T> {
// pub fn visit_logs_data(&mut self, ld: &LogsData) {
//     for rl in &ld.resource_logs {
//         self.visit_resource_logs(rl)
//     }
// }

// pub fn visit_resource_logs(&mut self, rl: &ResourceLogs) {
//     if let Some(res) = rl.resource.as_ref() {
//         self.visit_resource(res);
//     }
//     for sl in &rl.scope_logs {
//         self.visit_scope_logs(sl)
//     }
// }

// pub fn visit_resource(&mut self, _r: &Resource) {
//     // ...
// }

// pub fn visit_scope(&mut self, _s: &InstrumentationScope) {
//     // ...
// }

// pub fn visit_scope_logs(&mut self, sl: &ScopeLogs) {
//     if let Some(scope) = sl.scope.as_ref() {
//         self.visit_scope(scope);
//     }
//     for lr in &sl.log_records {
//         self.visit_log_record(lr)
//     }
// }

// pub fn visit_log_record(&mut self, _l: &LogRecord) {
//     self.0 += 1;
// }
//}

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

        let ic = LogsAdapter::new(&ld).visit(ItemCounter::new());
        assert_eq!(2, ic.0);
    }
}
