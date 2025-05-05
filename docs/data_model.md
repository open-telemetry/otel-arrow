# Arrow Data Model

This document describes the Arrow Schema used for each OTLP entity as Entity
Relation diagrams.

The Arrow Data Model has been carefully designed to optimize:

- The compression ratio for metrics, logs, and traces,
- Its compatibility within the extensive Arrow ecosystem,
- Its compatibility with file formats, such as Parquet.

This document has been generated directly from the source code. To regenerate
this document, run the following command:

```bash
make doc
```

## Metrics Arrow Records

The following ER diagram describes the Arrow Schema used for metrics.

```mermaid
erDiagram
    METRICS ||--o{ RESOURCE_ATTRS : resource-attrs
    METRICS ||--o{ SCOPE_ATTRS : scope-attrs
    METRICS ||--o{ NUMBER_DATA_POINTS : number-dps
    NUMBER_DATA_POINTS ||--o{ NUMBER_DP_ATTRS : number-dp-attrs
    NUMBER_DATA_POINTS ||--o{ NUMBER_DP_EXEMPLARS : number-dp-exemplars
    NUMBER_DP_EXEMPLARS ||--o{ NUMBER_DP_EXEMPLAR_ATTRS : number-dp-exemplar-attrs
    METRICS ||--o{ SUMMARY_DATA_POINTS : summary-dps
    SUMMARY_DATA_POINTS ||--o{ quantile : quantile
    SUMMARY_DATA_POINTS ||--o{ SUMMARY_DP_ATTRS : summary-dp-attrs
    METRICS ||--o{ HISTOGRAM_DATA_POINTS : histogram-dps
    HISTOGRAM_DATA_POINTS ||--o{ HISTOGRAM_DP_ATTRS : histogram-dp-attrs
    HISTOGRAM_DATA_POINTS ||--o{ HISTOGRAM_DP_EXEMPLARS : histogram-dp-exemplars
    HISTOGRAM_DP_EXEMPLARS ||--o{ HISTOGRAM_DP_EXEMPLAR_ATTRS : histogram-dp-exemplar-attrs
    METRICS ||--o{ EXP_HISTOGRAM_DATA_POINTS : exp-histogram-dps
    EXP_HISTOGRAM_DATA_POINTS ||--o{ EXP_HISTOGRAM_DP_ATTRS : exp-histogram-dp-attrs
    EXP_HISTOGRAM_DATA_POINTS ||--o{ EXP_HISTOGRAM_DP_EXEMPLARS : exp-histogram-dp-exemplars
    EXP_HISTOGRAM_DP_EXEMPLARS ||--o{ EXP_HISTOGRAM_DP_EXEMPLAR_ATTRS : exp-histogram-dp-exemplar-attrs
    SCOPE_ATTRS{
        parent_id u16
        key string
        type u8
        str string
        int i64 "optional"
        double f64 "optional"
        bool bool "optional"
        bytes bytes "optional"
        ser bytes "optional"
    }
    NUMBER_DATA_POINTS{
        id u32
        parent_id u16
        start_time_unix_nano timestamp
        time_unix_nano timestamp
        int_value i64
        double_value f64
        flags u32 "optional"
    }
    NUMBER_DP_EXEMPLARS{
        id u32 "optional"
        parent_id u32
        time_unix_nano timestamp "optional"
        int_value i64 "optional"
        double_value f64 "optional"
        span_id bytes[8] "optional"
        trace_id bytes[16] "optional"
    }
    NUMBER_DP_EXEMPLAR_ATTRS{
        parent_id u32
        key string
        type u8
        str string
        int i64 "optional"
        double f64 "optional"
        bool bool "optional"
        bytes bytes "optional"
        ser bytes "optional"
    }
    HISTOGRAM_DATA_POINTS{
        id u32 "optional"
        parent_id u16
        start_time_unix_nano timestamp "optional"
        time_unix_nano timestamp "optional"
        count u64 "optional"
        sum f64 "optional"
        bucket_counts u64 "optional"
        explicit_bounds f64 "optional"
        flags u32 "optional"
        min f64 "optional"
        max f64 "optional"
    }
    EXP_HISTOGRAM_DP_ATTRS{
        parent_id u32
        key string
        type u8
        str string
        int i64 "optional"
        double f64 "optional"
        bool bool "optional"
        bytes bytes "optional"
        ser bytes "optional"
    }
    EXP_HISTOGRAM_DP_EXEMPLAR_ATTRS{
        parent_id u32
        key string
        type u8
        str string
        int i64 "optional"
        double f64 "optional"
        bool bool "optional"
        bytes bytes "optional"
        ser bytes "optional"
    }
    RESOURCE_ATTRS{
        parent_id u16
        key string
        type u8
        str string
        int i64 "optional"
        double f64 "optional"
        bool bool "optional"
        bytes bytes "optional"
        ser bytes "optional"
    }
    NUMBER_DP_ATTRS{
        parent_id u32
        key string
        type u8
        str string
        int i64 "optional"
        double f64 "optional"
        bool bool "optional"
        bytes bytes "optional"
        ser bytes "optional"
    }
    SUMMARY_DATA_POINTS{
        id u32 "optional"
        parent_id u16
        start_time_unix_nano timestamp "optional"
        time_unix_nano timestamp "optional"
        count u64 "optional"
        sum f64 "optional"
        flags u32 "optional"
    }
    quantile{
        quantile f64 "optional"
        value f64 "optional"
    }
    SUMMARY_DP_ATTRS{
        parent_id u32
        key string
        type u8
        str string
        int i64 "optional"
        double f64 "optional"
        bool bool "optional"
        bytes bytes "optional"
        ser bytes "optional"
    }
    METRICS{
        id u16
        resource_id u16 "optional"
        resource_schema_url string "optional"
        resource_dropped_attributes_count u32 "optional"
        scope_id u16 "optional"
        scope_name string "optional"
        scope_version string "optional"
        scope_dropped_attributes_count u32 "optional"
        schema_url string "optional"
        metric_type u8
        name string
        description string "optional"
        unit string "optional"
        aggregation_temporality i32 "optional"
        is_monotonic bool "optional"
    }
    HISTOGRAM_DP_ATTRS{
        parent_id u32
        key string
        type u8
        str string
        int i64 "optional"
        double f64 "optional"
        bool bool "optional"
        bytes bytes "optional"
        ser bytes "optional"
    }
    HISTOGRAM_DP_EXEMPLARS{
        id u32 "optional"
        parent_id u32
        time_unix_nano timestamp "optional"
        int_value i64 "optional"
        double_value f64 "optional"
        span_id bytes[8] "optional"
        trace_id bytes[16] "optional"
    }
    HISTOGRAM_DP_EXEMPLAR_ATTRS{
        parent_id u32
        key string
        type u8
        str string
        int i64 "optional"
        double f64 "optional"
        bool bool "optional"
        bytes bytes "optional"
        ser bytes "optional"
    }
    EXP_HISTOGRAM_DATA_POINTS{
        id u32 "optional"
        parent_id u16
        start_time_unix_nano timestamp "optional"
        time_unix_nano timestamp "optional"
        count u64 "optional"
        sum f64 "optional"
        scale i32 "optional"
        zero_count u64 "optional"
        positive_offset i32 "optional"
        positive_bucket_counts u64 "optional"
        negative_offset i32 "optional"
        negative_bucket_counts u64 "optional"
        flags u32 "optional"
        min f64 "optional"
        max f64 "optional"
    }
    EXP_HISTOGRAM_DP_EXEMPLARS{
        id u32 "optional"
        parent_id u32
        time_unix_nano timestamp "optional"
        int_value i64 "optional"
        double_value f64 "optional"
        span_id bytes[8] "optional"
        trace_id bytes[16] "optional"
    }
```

## Logs Arrow Records

The following ER diagram describes the Arrow Schema used for logs.

```mermaid
erDiagram
    LOGS ||--o{ RESOURCE_ATTRS : resource-attrs
    LOGS ||--o{ SCOPE_ATTRS : scope-attrs
    LOGS ||--o{ LOG_ATTRS : logs-attrs
    LOGS{
        id u16 "optional"
        resource_id u16 "optional"
        resource_schema_url string "optional"
        resource_dropped_attributes_count u32 "optional"
        scope_id u16 "optional"
        scope_name string "optional"
        scope_version string "optional"
        scope_dropped_attributes_count u32 "optional"
        schema_url string "optional"
        time_unix_nano timestamp
        observed_time_unix_nano timestamp
        trace_id bytes[16] "optional"
        span_id bytes[8] "optional"
        severity_number i32 "optional"
        severity_text string "optional"
        body_type u8
        body_str string
        body_int i64 "optional"
        body_double f64 "optional"
        body_bool bool "optional"
        body_bytes bytes "optional"
        body_ser bytes "optional"
        dropped_attributes_count u32 "optional"
        flags u32 "optional"
    }
    RESOURCE_ATTRS{
        parent_id u16
        key string
        type u8
        str string
        int i64 "optional"
        double f64 "optional"
        bool bool "optional"
        bytes bytes "optional"
        ser bytes "optional"
    }
    SCOPE_ATTRS{
        parent_id u16
        key string
        type u8
        str string
        int i64 "optional"
        double f64 "optional"
        bool bool "optional"
        bytes bytes "optional"
        ser bytes "optional"
    }
    LOG_ATTRS{
        parent_id u16
        key string
        type u8
        str string
        int i64 "optional"
        double f64 "optional"
        bool bool "optional"
        bytes bytes "optional"
        ser bytes "optional"
    }
```

## Traces Arrow Records

The following ER diagram describes the Arrow Schema used for traces.

```mermaid
erDiagram
    SPANS ||--o{ RESOURCE_ATTRS : resource-attrs
    SPANS ||--o{ SCOPE_ATTRS : scope-attrs
    SPANS ||--o{ SPAN_ATTRS : span-attrs
    SPANS ||--o{ SPAN_EVENTS : span-event
    SPANS ||--o{ SPAN_LINKS : span-link
    SPAN_EVENTS ||--o{ SPAN_EVENT_ATTRS : span-event-attrs
    SPAN_LINKS ||--o{ SPAN_LINK_ATTRS : span-link-attrs
    SPANS{
        id u16 "optional"
        resource_id u16 "optional"
        resource_schema_url string "optional"
        resource_dropped_attributes_count u32 "optional"
        scope_id u16 "optional"
        scope_name string "optional"
        scope_version string "optional"
        scope_dropped_attributes_count u32 "optional"
        schema_url string "optional"
        start_time_unix_nano timestamp
        duration_time_unix_nano duration
        trace_id bytes[16]
        span_id bytes[8]
        trace_state string "optional"
        parent_span_id bytes[8] "optional"
        name string
        kind i32 "optional"
        dropped_attributes_count u32 "optional"
        dropped_events_count u32 "optional"
        dropped_links_count u32 "optional"
        status_code i32 "optional"
        status_status_message string "optional"
    }
    RESOURCE_ATTRS{
        parent_id u16
        key string
        type u8
        str string
        int i64 "optional"
        double f64 "optional"
        bool bool "optional"
        bytes bytes "optional"
        ser bytes "optional"
    }
    SCOPE_ATTRS{
        parent_id u16
        key string
        type u8
        str string
        int i64 "optional"
        double f64 "optional"
        bool bool "optional"
        bytes bytes "optional"
        ser bytes "optional"
    }
    SPAN_ATTRS{
        parent_id u16
        key string
        type u8
        str string
        int i64 "optional"
        double f64 "optional"
        bool bool "optional"
        bytes bytes "optional"
        ser bytes "optional"
    }
    SPAN_EVENTS{
        id u32 "optional"
        parent_id u16
        time_unix_nano timestamp "optional"
        name string
        dropped_attributes_count u32 "optional"
    }
    SPAN_LINKS{
        id u32 "optional"
        parent_id u16
        trace_id bytes[16] "optional"
        span_id bytes[8] "optional"
        trace_state string "optional"
        dropped_attributes_count u32 "optional"
    }
    SPAN_EVENT_ATTRS{
        parent_id u32
        key string
        type u8
        str string
        int i64 "optional"
        double f64 "optional"
        bool bool "optional"
        bytes bytes "optional"
        ser bytes "optional"
    }
    SPAN_LINK_ATTRS{
        parent_id u32
        key string
        type u8
        str string
        int i64 "optional"
        double f64 "optional"
        bool bool "optional"
        bytes bytes "optional"
        ser bytes "optional"
    }
```
