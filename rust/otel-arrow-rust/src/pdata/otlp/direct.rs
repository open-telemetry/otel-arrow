use crate::proto::opentelemetry::common::v1::InstrumentationScope;
use crate::proto::opentelemetry::logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs};
use crate::proto::opentelemetry::resource::v1::Resource;
// use crate::proto::opentelemetry::common::v1::KeyValue;
// use crate::proto::opentelemetry::common::v1::KeyValueList;

pub struct ItemCounter(usize);

impl ItemCounter {
    pub fn new() -> Self {
        Self(0)
    }

    pub fn visit_logs_data(&mut self, ld: &LogsData) {
        for rl in &ld.resource_logs {
            self.visit_resource_logs(rl)
        }
    }

    pub fn visit_resource_logs(&mut self, rl: &ResourceLogs) {
        if let Some(res) = rl.resource.as_ref() {
            self.visit_resource(res);
        }
        for sl in &rl.scope_logs {
            self.visit_scope_logs(sl)
        }
    }

    pub fn visit_resource(&mut self, _r: &Resource) {
        // ...
    }

    pub fn visit_scope(&mut self, _s: &InstrumentationScope) {
        // ...
    }

    pub fn visit_scope_logs(&mut self, sl: &ScopeLogs) {
        if let Some(scope) = sl.scope.as_ref() {
            self.visit_scope(scope);
        }
        for lr in &sl.log_records {
            self.visit_log_record(lr)
        }
    }

    pub fn visit_log_record(&mut self, _l: &LogRecord) {
        self.0 += 1;
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

        ic.visit_logs_data(&ld);
        assert_eq!(2, ic.0);
    }
}

// trait Visitable {
// //    type Return;
// }

// trait VisitLogsData<T>
// where
//     T: Visitable,
// {
//     fn visit_resource_logs(v: T) -> impl VisitResourceLogs<T>;
// }

// trait VisitResourceLogs<T>
// where
//     T: Visitable,
// {
//     // fn visit_resource(v: T) -> impl VisitResource<T::Return0>;
//     // fn visit_scope_logs(v: T) -> impl VisitScopeLogs<T::Return1>;
// }

// // trait VisitResource {
// // }

// // trait VisitScopeLogs {
// // }

// impl ItemCounter {

// }

// impl Visit

// trait AnyValueVisitor {
//     fn visit_string(&mut self) {
//     }
//     fn visit_i64(&mut self) {
//     }
//     fn visit_f64(&mut self) {
//     }
// };

// // ...
// trait Visitor<R>
// {
//     fn visit(&self);
// }

// impl Visitor<T> for LogsData {
//     fn visit(&self, r: R) -> R;
// };

// trait LogsDataVisitor<R> {
//     fn visit_resource_logs(&mut self, r: mut R) -> R;
// }

// trait ResourceLogsVisitor<R>: ResourceVisitor<R> {
//     fn visit_scope_logs(& self, r: mut R) -> R;
// }

// trait ResourceVisitor<R> {
//     fn visit_resource(&mut self, r: mut R) -> R;
// }

// trait ScopeLogsVisitor<R> {
//     fn visit_scope(&mut self, r: mut R) -> R;
// }

// trait ScopeVisitor<R>: LogRecordVisitor {
//     fn visit_scope(&mut self, r: mut R) -> R;
// }

// trait LogRecordsVisitor<R> {
//     fn visit_log_record(&mut self, r: mut R) -> R;
// }

// trait VisitIdx<T, R> {
//     fn visit_idx(&mut self, item: T) -> R;
// }

// struct SliceVisitor {
//     items: Iterator
// }

// //trait VisitSlice<T, R> {
// //    fn visit_slice(&mut self, item: &AsRef<[T]>) -> R;
// //}

// struct<T> Count<T> {
//     cnt: T,
// }

//impl VisitIdx<T> f

// impl<T, R> SliceVisitor<T, R> for Count<R> {
//     fn visit_item(&mut self, _: T) {
// 	self.cnt += 1;
//     }
// }

// struct SliceVisitor<S, T, R> {
//     state: S,

//     _tph PhantomData<T>,
//     _tph PhantomData<R>,
// }

// impl<S, T, R> SliceVisitor<S, T, R> {
//     fn visit(slice: AsRef<[T]>) -> R {
// 	for item in slice.as_ref() {
// 	    visit(
// 	}
//     }
// }

// struct ItemVisitor<S, T, R> {
//     state: S,

//     _tph PhantomData<T>,
//     _tph PhantomData<R>,
// }

// impl<S, T, R> ItemVisitor<S, T, R> {
//     fn visit(item: AsRef<T>) -> R {
// 	for item in item.as_ref() {
// 	    visit(
// 	}
//     }
// }

// struct KeyValueVisitor {

// }

// struct Sizer {

// }

// struct Marshaler {
//     //fn marshal_to()
// }
