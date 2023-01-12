# Arrow Schema

This document describes the Arrow Schema used for each OTLP entity.

## Principles

- Each type of OTLP entity is described by a specific Arrow Schema. 
- Attributes, values and other dynamic fields are represented by Arrow type `union` to facilitate the creation and 
interpretation of Arrow records.
- Since there is no official Arrow Schema format, this document describes them in a simple and self-explanatory YAML 
format.
- Repetitive declaration (e.g. attributes) are referenced using YAML anchors (&) and aliases (*).

## Attributes Arrow Schema

Attributes are represented as a map of key-value pairs. The key is a dictionary encoded string, and the value is a
sparse union of string, int64, float64, bool, and binary. The dictionary encoding is used to reduce the size of the
payload. The sparse union is used to represent the value type in a flexible and extensible way.

```yaml
attributes: &attributes                 # arrow map
  key: string_dictionary | string       # string_dictionary by default, fallback to string when cardinality too high
  value:                                # arrow sparse union
    str: string_dictionary | string     # string_dictionary by default, fallback to string when cardinality too high
    i64: int64
    f64: float64
    bool: bool
    binary: binary_dictionary | binary  # binary_dictionary by default, fallback to binary when cardinality too high
    cbor: binary_dictionary | binary    # binary_dictionary by default, fallback to binary when cardinality too high
---

exemplars: &exemplars                                       # arrow list of 
  - attributes: *attributes                                 # arrow map
    time_unix_nano: timestamp                               # time unit nanoseconds
    value:                                                  # arrow sparse union
      i64: int64
      f64: float64
    span_id: 8_bytes_binary_dictionary | 8_bytes_binary     # arrow fixed size binary array, 8_bytes_binary_dictionary by default, fallback to 8_bytes_binary when cardinality too high
    trace_id: 16_bytes_binary_dictionary | 16_bytes_binary  # arrow fixed size binary array, 16_bytes_binary_dictionary by default, fallback to 16_bytes_binary when cardinality too high
---

resource_metrics:                                                       # arrow list of 
  - resource:                                                           # struct
      attributes: *attributes                                           # arrow map
      dropped_attributes_count: uint32
    schema_url: string_dictionary | string                              # string_dictionary by default, fallback to string when cardinality too high
    scope_metrics:                                                      # arrow list of 
      - scope:                                                          # struct
          name: string_dictionary | string                              # string_dictionary by default, fallback to string when cardinality too high
          version: string_dictionary | string                           # string_dictionary by default, fallback to string when cardinality too high
          attributes: *attributes                                       # arrow map
          dropped_attributes_count: uint32
        schema_url: string_dictionary | string                          # string_dictionary by default, fallback to string when cardinality too high
        univariate_metrics:                                             # arrow list of 
          - name: string_dictionary | string                            # string_dictionary by default, fallback to string when cardinality too high
            description: string_dictionary | string                     # string_dictionary by default, fallback to string when cardinality too high
            unit: string_dictionary | string                            # string_dictionary by default, fallback to string when cardinality too high
            data:                                                       # arrow sparse union
              gauge:                                                    # struct
                data_points:                                            # arrow list of 
                  - attributes: *attributes                             # arrow map
                    start_time_unix_nano: timestamp                     # time unit nanoseconds
                    time_unix_nano: timestamp                           # time unit nanoseconds
                    value:                                              # arrow sparse union
                      i64: int64
                      f64: float64
                    exemplars: *exemplars                               # arrow list of 
                    flags: uint32                                       # used as a bit mask
              sum:                                                      # struct
                data_points:                                            # arrow list of 
                  - attributes: *attributes                             # arrow map
                    start_time_unix_nano: timestamp                     # time unit nanoseconds
                    time_unix_nano: timestamp                           # time unit nanoseconds
                    value:                                              # arrow sparse union
                      i64: int64
                      f64: float64
                    exemplars: *exemplars                               # arrow list of 
                    flags: uint32                                       # used as a bit mask
                aggregation_temporality: int32_dictionary | int32       # int32_dictionary by default, fallback to int32 when cardinality too high, OTLP enum with 3 variants
                is_monotonic: bool
              summary:                                                  # struct
                data_points:                                            # arrow list of 
                  - attributes: *attributes                             # arrow map
                    start_time_unix_nano: timestamp                     # time unit nanoseconds
                    time_unix_nano: timestamp                           # time unit nanoseconds
                    count: uint64
                    sum: float64
                    quantile:                                           # arrow list of 
                      - quantile: float64
                        value: float64
                    flags: uint32                                       # used as a bit mask
              histogram:                                                # struct
                data_points:                                            # arrow list of 
                  - attributes: *attributes                             # arrow map
                    start_time_unix_nano: timestamp                     # time unit nanoseconds
                    time_unix_nano: timestamp                           # time unit nanoseconds
                    count: uint64
                    sum: float64
                    bucket_counts:                                      # arrow list of uint64
                    explicit_bounds:                                    # arrow list of float64
                    exemplars: *exemplars                               # arrow list of 
                    flags: uint32                                       # used as a bit mask
                    min: float64
                    max: float64
                aggregation_temporality: int32_dictionary | int32       # int32_dictionary by default, fallback to int32 when cardinality too high, OTLP enum with 3 variants
              exp_histogram:                                            # struct
                data_points:                                            # arrow list of 
                  - attributes: *attributes                             # arrow map
                    start_time_unix_nano: timestamp                     # time unit nanoseconds
                    time_unix_nano: timestamp                           # time unit nanoseconds
                    count: uint64
                    sum: float64
                    scale: int32
                    zero_count: uint64
                    positive:                                           # struct
                      offset: int32
                      bucket_counts:                                    # arrow list of uint64
                    negative:                                           # struct
                      offset: int32
                      bucket_counts:                                    # arrow list of uint64
                    exemplars: *exemplars                               # arrow list of 
                    flags: uint32                                       # used as a bit mask
                    min: float64
                    max: float64
                aggregation_temporality: int32_dictionary | int32       # int32_dictionary by default, fallback to int32 when cardinality too high, OTLP enum with 3 variants
            shared_attributes: *attributes                              # arrow map, inherited by data points
            shared_start_time_unix_nano: timestamp                      # time unit nanoseconds, inherited by data points
            shared_time_unix_nano: timestamp                            # time unit nanoseconds, inherited by data points
        shared_attributes: *attributes                                  # arrow map, inherited by data points
        shared_start_time_unix_nano: timestamp                          # time unit nanoseconds, inherited by data points
        shared_time_unix_nano: timestamp                                # time unit nanoseconds, inherited by data points
---

resource_logs:                                                          # arrow list of 
  - resource:                                                           # struct
      attributes: *attributes                                           # arrow map
      dropped_attributes_count: uint32
    schema_url: string_dictionary | string                              # string_dictionary by default, fallback to string when cardinality too high
    scope_logs:                                                         # arrow list of 
      - scope:                                                          # struct
          name: string_dictionary | string                              # string_dictionary by default, fallback to string when cardinality too high
          version: string_dictionary | string                           # string_dictionary by default, fallback to string when cardinality too high
          attributes: *attributes                                       # arrow map
          dropped_attributes_count: uint32
        schema_url: string_dictionary | string                          # string_dictionary by default, fallback to string when cardinality too high
        logs:                                                           # arrow list of 
          - time_unix_nano: timestamp                                   # time unit nanoseconds
            observed_time_unix_nano: timestamp                          # time unit nanoseconds
            trace_id: 16_bytes_binary_dictionary | 16_bytes_binary      # arrow fixed size binary array, 16_bytes_binary_dictionary by default, fallback to 16_bytes_binary when cardinality too high
            span_id: 8_bytes_binary_dictionary | 8_bytes_binary         # arrow fixed size binary array, 8_bytes_binary_dictionary by default, fallback to 8_bytes_binary when cardinality too high
            severity_number: int32_dictionary | int32                   # int32_dictionary by default, fallback to int32 when cardinality too high, OTLP enum with 25 variants
            severity_text: string_dictionary | string                   # string_dictionary by default, fallback to string when cardinality too high
            body:                                                       # arrow sparse union
              str: string_dictionary | string                           # string_dictionary by default, fallback to string when cardinality too high
              i64: int64
              f64: float64
              bool: bool
              binary: binary_dictionary | binary                        # binary_dictionary by default, fallback to binary when cardinality too high
              cbor: binary_dictionary | binary                          # binary_dictionary by default, fallback to binary when cardinality too high
            attributes: *attributes                                     # arrow map
            dropped_attributes_count: uint32
            flags: uint32                                               # used as a bit mask
---

resource_spans:                                                         # arrow list of 
  - resource:                                                           # struct
      attributes: *attributes                                           # arrow map
      dropped_attributes_count: uint32
    schema_url: string_dictionary | string                              # string_dictionary by default, fallback to string when cardinality too high
    scope_spans:                                                        # arrow list of 
      - scope:                                                          # struct
          name: string_dictionary | string                              # string_dictionary by default, fallback to string when cardinality too high
          version: string_dictionary | string                           # string_dictionary by default, fallback to string when cardinality too high
          attributes: *attributes                                       # arrow map
          dropped_attributes_count: uint32
        schema_url: string_dictionary | string                          # string_dictionary by default, fallback to string when cardinality too high
        spans:                                                          # arrow list of 
          - start_time_unix_nano: timestamp                             # time unit nanoseconds
            end_time_unix_nano: timestamp                               # time unit nanoseconds
            trace_id: 16_bytes_binary_dictionary | 16_bytes_binary      # arrow fixed size binary array, 16_bytes_binary_dictionary by default, fallback to 16_bytes_binary when cardinality too high
            span_id: 8_bytes_binary_dictionary | 8_bytes_binary         # arrow fixed size binary array, 8_bytes_binary_dictionary by default, fallback to 8_bytes_binary when cardinality too high
            trace_state: string_dictionary | string                     # string_dictionary by default, fallback to string when cardinality too high
            parent_span_id: 8_bytes_binary_dictionary | 8_bytes_binary  # arrow fixed size binary array, 8_bytes_binary_dictionary by default, fallback to 8_bytes_binary when cardinality too high
            name: string_dictionary | string                            # string_dictionary by default, fallback to string when cardinality too high
            kind: int32_dictionary | int32                              # int32_dictionary by default, fallback to int32 when cardinality too high, OTLP enum with 6 variants
            attributes: *attributes                                     # arrow map
            dropped_attributes_count: uint32
            events:                                                     # arrow list of 
              - time_unix_nano: timestamp                               # time unit nanoseconds
                name: string_dictionary | string                        # string_dictionary by default, fallback to string when cardinality too high
                attributes: *attributes                                 # arrow map
                dropped_attributes_count: uint32
            dropped_events_count: uint32
            links:                                                      # arrow list of 
              - trace_id: 16_bytes_binary_dictionary | 16_bytes_binary  # arrow fixed size binary array, 16_bytes_binary_dictionary by default, fallback to 16_bytes_binary when cardinality too high
                span_id: 8_bytes_binary_dictionary | 8_bytes_binary     # arrow fixed size binary array, 8_bytes_binary_dictionary by default, fallback to 8_bytes_binary when cardinality too high
                trace_state: string_dictionary | string                 # string_dictionary by default, fallback to string when cardinality too high
                attributes: *attributes                                 # arrow map
                dropped_attributes_count: uint32
            dropped_links_count: uint32
            status:                                                     # struct
              code: int32_dictionary | int32                            # int32_dictionary by default, fallback to int32 when cardinality too high, OTLP enum with 4 variants
              status_message: string_dictionary | string                # string_dictionary by default, fallback to string when cardinality too high
---
```

This schema has been generated directly from the source code. To regenerate this schema, run the following command:

```bash
go run tools/doc_schema_gen/main.go
``` 