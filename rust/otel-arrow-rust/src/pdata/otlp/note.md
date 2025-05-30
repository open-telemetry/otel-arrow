## Visitor pattern

The pattern is to define two traits per OTLP data type .  For
productionizing this concept, we will extend the ./derive macro
package.

For each type there is a **Visitor** (an actor) and a
**Visitable**. Using these traits allows an implementation to perform
an in-order traversal of OTLP-like data structure.

A Visitable impl is passed to the Visitor to enact the
visitation. Visitable adapter structs are provided for presenting OTLP
data types as Visitable impls. Visitable impls are immutable, passed
by `&self`.

Visitor impls are passed to the Visitable carrying logic as the
traversal descends into a child. Visitors are provided for each field
of the child. Visitors are mutable, passed by `&mut self`.

In general, the Visitor calls the Visitable and the Visitable calls
the child Visitors. At the top-level, users are directed to use for
example, LogsVisitor which consumes the visitor and returns the
associated **Return** type.

As an example, to count log records we can use the visitor:

```
pub fn LogRecordCount(ld: &LogsData) -> usize {
    ItemCounter::new().visit_logs(&LogsDataAdapter::new(&ld))
}
```

Benchmark results for counting 10 resources * 10 scopes each * 10
records each indicate a slowdown of 50%.

```
OTLP Logs counting/Visitor
                        time:   [1.4456 ns 1.4878 ns 1.5354 ns]
OTLP Logs counting/Manual
                        time:   [930.08 ps 962.65 ps 996.07 ps]
```

This is inconclusive. We expect some cost for the abstraction, and a
more realistic application should be measured.

## Example traits

For the `LogsData` type, which is a top-level container for logs, the
Visitor trait is:

```
pub trait LogsDataVisitor {
    fn visit_logs_data(&mut self, v: impl LogsDataVisitable);
}
```

The Vistable trait is:

```
pub trait LogsDataVisitable {
    fn visit_logs_data(&self, v: impl ResourceLogsVisitor);
}
```

The OTLP adapter is:

```
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
```

and the complete log item-counter implementation used for the benchmark:

```
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
```

## Next steps

Implement a OTLP protocol buffer encoding using this abstraction. The
same abstraction will be implemented for OTAP data frames, and we
expect that directly encoding OTLP bytes using the visitor will
outperform the use of intermediate protocol buffer objects for
encoding.

A two-pass implementation will:

1. Using scratch memory, build a Vec<usize> containing the size of
   every length-delimited field in traversal order.
2. Generate output in a second pass while iterating over the
   Vec<usize> above to know the size of each object in advance.

We recognize that this is not the only way to encode a protocol
buffer. Our task is to benchmark this algorithm and compare it with
the performance of directly encoding with Prost.
