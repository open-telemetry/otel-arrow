# Debug Exporter

Status: **WIP**

This crate will contain the implementation of the debug exporter.

## Example Output => Basic Verbosity

```plaintext
Timer tick received
OTLP Metric objects received: 0
OTLP Trace objects received: 0
OTLP Profile objects received: 0
OTLP Log objects received: 0
Received 1 resource metrics
Received 5 metrics
Received 5 data points
Received 1 resource spans
Received 1 spans
Received 1 events
Received 1 links
Received 1 resource logs
Received 1 log records
Received 1 events
Received 1 resource profiles
Received 0 samples
Shutdown message received
Debug Exporter Summary:
OTLP Metric objects received: 1
Received 1 Resource Metrics
Received 5 metrics
Received 5 datapoints
OTLP Trace objects received: 1
Received 1 Resource Spans
Received 1 spans
Received 1 events
Received 1 links
OTLP Log objects received: 1
Received 1 Resource logs
Received 1 log records
Received 1 log events
OTLP Profile objects received: 1
Received 1 Resource profiles
Received 0 samples

```

## Example Output => Normal Verbosity

```plaintext
Timer tick received
OTLP Metric objects received: 0
OTLP Trace objects received: 0
OTLP Profile objects received: 0
OTLP Log objects received: 0
Received 1 resource metrics
Received 5 metrics
Received 5 data points
ResourceMetric #0, Schema:[http://schema.opentelemetry.io], Attributes: ip=192.168.0.2 
   ScopeMetric #0, Name: library, Version: @v1, Schema: [http://schema.opentelemetry.io], Attributes: instrumentation_scope_k1=k1 value 
      system.cpu.time 0
      system.cpu.time freq=3GHz count=0 sum=56 min=12 max=100.1 
      system.cpu.time freq=3GHz count=0 sum=56 min=12 max=100.1 le94.17542094619048=0 
      system.cpu.time cpu_logical_processors=8 0
      system.cpu.time cpu_cores=4 count=0 sum=56 q0=0 

Received 1 resource spans
Received 1 spans
Received 1 events
Received 1 links
ResourceSpan #0, Schema:[http://schema.opentelemetry.io], Attributes: ip=192.168.0.1 
   ScopeSpan #0, Name: library, Version: @v1, Schema: [http://schema.opentelemetry.io], Attributes: hostname=host5.retailer.com 
      Name: user-account, Trace ID: 4327e52011a22f9662eac217d77d1ec0, Span ID: 7271ee06d7e5925f, Attributes: hostname=host4.gov 

Received 1 resource logs
Received 1 log records
Received 1 events
ResourceLog #0, Schema:[http://schema.opentelemetry.io], Attributes: version=2.0 
   ScopeLog #0, Name: library, Version: @v1, Schema: [http://schema.opentelemetry.io], Attributes: hostname=host5.retailer.com 
      Body: Sint impedit non ut eligendi nisi neque harum maxime adipisci., Attributes: hostname=host3.thedomain.edu 

Received 1 resource profiles
Received 0 samples
ResourceProfile #0, Schema:[http://schema.opentelemetry.io], Attributes: hostname=host7.com 
   ScopeProfile #0, Name: library, Version: @v1, Schema: [http://schema.opentelemetry.io], Attributes: hostname=host5.retailer.com 

Shutdown message received
Debug Exporter Summary:
OTLP Metric objects received: 1
Received 1 Resource Metrics
Received 5 metrics
Received 5 datapoints
OTLP Trace objects received: 1
Received 1 Resource Spans
Received 1 spans
Received 1 events
Received 1 links
OTLP Log objects received: 1
Received 1 Resource logs
Received 1 log records
Received 1 log events
OTLP Profile objects received: 1
Received 1 Resource profiles
Received 0 samples
```

## Example Output => Detailed Verbosity

```plaintext
Timer tick received
OTLP Metric objects received: 0
OTLP Trace objects received: 0
OTLP Profile objects received: 0
OTLP Log objects received: 0
Received 1 resource metrics
Received 5 metrics
Received 5 data points
ResourceMetric #0
   -> Resource SchemaURL: http://schema.opentelemetry.io
   -> Resource Attributes:
         -> ip: 192.168.0.2
   ScopeMetrics #0
      -> ScopeMetrics SchemaURL: http://schema.opentelemetry.io
      -> Instrumentation Scope library @v1
      -> Instrumentation Scope Attributes:
         -> instrumentation_scope_k1: k1 value
      Metric #0
         -> Name: system.cpu.time
         -> Description: time cpu has ran
         -> Unit: s
         -> DataType: Gauge
            NumberDataPoints #0
               -> Attributes:
               -> StartTimestamp: 1650499200000000100
               -> Timestamp: 1663718400000001400
               -> Value: 0
      Metric #1
         -> Name: system.cpu.time
         -> Description: time cpu has ran
         -> Unit: s
         -> DataType: Exponential Histogram
         -> AggregationTemporality: 4
            ExponentialHistogramDataPoints #0
               -> Attributes:
                     -> freq: 3GHz
               -> StartTimestamp: 1650499200000000000
               -> Timestamp: 1663718400000001400
               -> Count: 0
               -> Sum: 56
               -> Min: 12
               -> Max: 100.1
               -> Bucket [-4.113250378782927, -1), Count: 0
               -> Bucket (1, 4.113250378782927], Count: 0
               -> Exemplars:
                  Exemplar #0
                     -> Trace ID: 4327e52011a22f9662eac217d77d1ec0
                     -> Span ID: 7271ee06d7e5925f
                     -> Timestamp: 1663718400000001400
                     -> Value: 22.2
                     -> FilteredAttributes:
                        -> cpu: 0
      Metric #2
         -> Name: system.cpu.time
         -> Description: time cpu has ran
         -> Unit: s
         -> DataType: Histogram
         -> AggregationTemporality: 4
            HistogramDataPoints #0
               -> Attributes:
                     -> freq: 3GHz
               -> StartTimestamp: 1650499200000000000
               -> Timestamp: 1663718400000001400
               -> Count: 0
               -> Sum: 56
               -> Min: 12
               -> Max: 100.1
               -> ExplicitBound #0: 94.17542094619048
               -> ExplicitBound #1: 65.66722851519177
               -> Buckets #0, Count: 0
               -> Exemplars:
                  Exemplar #0
                     -> Trace ID: 4327e52011a22f9662eac217d77d1ec0
                     -> Span ID: 7271ee06d7e5925f
                     -> Timestamp: 1663718400000001400
                     -> Value: 22.2
                     -> FilteredAttributes:
                        -> cpu: 0
      Metric #3
         -> Name: system.cpu.time
         -> Description: time cpu has ran
         -> Unit: s
         -> DataType: Sum
         -> IsMonotonic: true
         -> AggregationTemporality: 4
            NumberDataPoints #0
               -> Attributes:
                     -> cpu_logical_processors: 8
               -> StartTimestamp: 1650499200000000000
               -> Timestamp: 1663718400000001400
               -> Value: 0
               -> Exemplars:
                  Exemplar #0
                     -> Trace ID: 4327e52011a22f9662eac217d77d1ec0
                     -> Span ID: 7271ee06d7e5925f
                     -> Timestamp: 1663718400000001400
                     -> Value: 22.2
                     -> FilteredAttributes:
                        -> ************: true
      Metric #4
         -> Name: system.cpu.time
         -> Description: time cpu has ran
         -> Unit: s
         -> DataType: Summary
            SummaryDataPoints #0
               -> Attributes:
                     -> cpu_cores: 4
               -> StartTimestamp: 1650499200000000100
               -> Timestamp: 1663718400000001400
               -> Count: 0
               -> Sum: 56
               -> QuantileValue #0: Quantile 0, Value 0

Received 1 resource spans
Received 1 spans
Received 1 events
Received 1 links
ResourceSpan #0
   -> Resource SchemaURL: http://schema.opentelemetry.io
   -> Resource Attributes:
      -> ip: 192.168.0.1
   ScopeSpans #0
      -> ScopeSpans SchemaURL: http://schema.opentelemetry.io
      -> Instrumentation Scope library @v1
      -> Instrumentation Scope Attributes:
         -> hostname: host5.retailer.com
      Span #0
         -> Trace ID: 4327e52011a22f9662eac217d77d1ec0
         -> Parent ID: 7271ee06d7e5925f
         -> ID: 7271ee06d7e5925f
         -> Name: user-account
         -> Kind: 4
         -> TraceState: ended
         -> Start time: 1647648000000000106
         -> End time: 1647648000000000104
         -> Status code: 2
         -> Status message: Error
         -> Attributes:
            -> hostname: host4.gov
         -> Events:
            SpanEvent #0
               -> Name: message-receive
               -> Timestamp: 1647648000000000108
               -> DroppedAttributesCount: 0
               -> Attributes:
                  -> hostname: host5.retailer.com
         -> Links:
            SpanLink: #0
               -> Trace ID: 4327e52011a22f9662eac217d77d1ec0
               -> Span ID: 7271ee06d7e5925f
               -> TraceState: ended
               -> DroppedAttributesCount: 0
               -> Attributes:
                  -> hostname: host2.org

Received 1 resource logs
Received 1 log records
Received 1 events
ResourceLog #0
   -> Resource SchemaURL: http://schema.opentelemetry.io
   -> Resource Attributes:
      -> version: 2.0
   ScopeLogs #0
      -> ScopeLogs SchemaURL: http://schema.opentelemetry.io
      -> Instrumentation Scope library @v1
      -> Instrumentation Scope Attributes:
         -> hostname: host5.retailer.com
      LogRecord #0
         -> ObservedTimestamp: 1663718400000001300
         -> Timestamp: 2000000000
         -> SeverityText: INFO
         -> SeverityNumber: 2
         -> EventName: event1
         -> Body: Sint impedit non ut eligendi nisi neque harum maxime adipisci.
         -> Attributes:
            -> hostname: host3.thedomain.edu
         -> Trace ID: 4327e52011a22f9662eac217d77d1ec0
         -> Span ID: 7271ee06d7e5925f
         -> Flags: 8

Received 1 resource profiles
Received 0 samples
ResourceProfile #0
   -> Resource SchemaURL: http://schema.opentelemetry.io
   -> Resource Attributes:
      -> hostname: host7.com
   ScopeProfiles #0
      -> ScopeProfiles SchemaURL: http://schema.opentelemetry.io
      -> Instrumentation Scope library @v1
      -> Instrumentation Scope Attributes:
         -> hostname: host5.retailer.com
      Profile #0
         -> Profile ID: 
         -> Start time: 0
         -> Duration: 0
         -> Dropped attributes count: 0
         -> Location indices: []

Shutdown message received
Debug Exporter Summary:
OTLP Metric objects received: 1
Received 1 Resource Metrics
Received 5 metrics
Received 5 datapoints
OTLP Trace objects received: 1
Received 1 Resource Spans
Received 1 spans
Received 1 events
Received 1 links
OTLP Log objects received: 1
Received 1 Resource logs
Received 1 log records
Received 1 log events
OTLP Profile objects received: 1
Received 1 Resource profiles
Received 0 samples

```