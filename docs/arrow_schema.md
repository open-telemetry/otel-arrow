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
payload. The sparse union is used to represent the value type.

```yaml
# Attributes Arrow Schema (declaration used in other schemas)
attributes: &attributes                                 # arrow type = map
  - key: string | string_dictionary
    value:                                              # arrow type = sparse union
        str: string | string_dictionary 
        i64: int64
        f64: float64
        bool: bool 
        binary: binary | binary_dictionary


# Metrics Arrow Schema
# OTLP univariate metrics are represented with the following Arrow Schema.
# TODO: Histogram, Exponential Histogram, Multivariate metrics

resource_metrics:
    - resource: 
        attributes: *attributes
        dropped_attributes_count: uint32 
      schema_url: string | string_dictionary
      scope_metrics: 
        - scope: 
            name: string | string_dictionary 
            version: string | string_dictionary 
            attributes: *attributes
            dropped_attributes_count: uint32
          schema_url: string | string_dictionary
          metrics: 
            - name: string | string_dictionary            # required
              description: string | string_dictionary
              unit: string | string_dictionary 
              data:                                       # arrow type = sparse union
                - gauge: 
                    data_points: 
                      - attributes: *attributes
                        start_time_unix_nano: uint64
                        time_unix_nano: uint64            # required
                        value:                            # arrow type = dense union
                          i64: int64 
                          f64: float64 
                        exemplars: 
                          - attributes: *attributes
                            time_unix_nano: uint64
                            value:                        # arrow type = dense union
                              i64: int64
                              f64: float64
                            span_id: 8_bytes_binary | 8_bytes_binary_dictionary 
                            trace_id: 16_bytes_binary | 16_bytes_binary_dictionary 
                        flags: uint32
                  sum: 
                    data_points: 
                      - attributes: *attributes
                        start_time_unix_nano: uint64
                        time_unix_nano: uint64            # required
                        value:                            # arrow type = dense union
                          i64: int64
                          f64: float64
                        exemplars: 
                          - attributes: *attributes
                            time_unix_nano: uint64
                            value:                        # arrow type = dense union
                              i64: int64
                              f64: float64
                            span_id: 8_bytes_binary | 8_bytes_binary_dictionary
                            trace_id: 16_bytes_binary | 16_bytes_binary_dictionary
                        flags: uint32
                    aggregation_temporality: int32
                    is_monotonic: bool
                  summary: 
                    data_points: 
                      - attributes: *attributes
                        start_time_unix_nano: uint64
                        time_unix_nano: uint64            # required
                        count: uint64
                        sum: float64
                        quantile: 
                          - quantile: float64
                            value: float64
                        flags: uint32


# Logs Arrow Schema
# OTLP Logs are represented with the following Arrow Schema.

resource_logs: 
  - resource: 
      attributes: *attributes
      dropped_attributes_count: uint32
    schema_url: string | string_dictionary 
    scope_logs: 
      - scope:
          name: string | string_dictionary 
          version: string | string_dictionary 
          attributes: *attributes
          dropped_attributes_count: uint32
        schema_url: string | string_dictionary 
        logs: 
          - time_unix_nano: uint64 
            observed_time_unix_nano: uint64 
            trace_id: 16_bytes_binary | 16_bytes_binary_dictionary 
            span_id: 8_bytes_binary | 8_bytes_binary_dictionary
            severity_number: int32 
            severity_text: string | string_dictionary 
            body:                                           # arrow type: sparse union
              str: string | string_dictionary 
              i64: int64 
              f64: float64 
              bool: bool 
              binary: binary | binary_dictionary
            attributes: *attributes
            dropped_attributes_count: uint32 
            flags: uint32

            
# Traces Arrow Schema
# OTLP Traces are represented with the following Arrow Schema.

resource_spans:
  - resource: 
      attributes: *attributes
      dropped_attributes_count: uint32
    schema_url: string | string_dictionary 
    scope_spans: 
      - scope: 
          name: string | string_dictionary 
          version: string | string_dictionary 
          attributes: *attributes
          dropped_attributes_count: uint32
        schema_url: string | string_dictionary 
        spans:
          - start_time_unix_nano: uint64                                  # required 
            end_time_unix_nano: uint64                                    # required
            trace_id: 16_bytes_binary | 16_bytes_binary_dictionary        # required
            span_id: 8_bytes_binary | 8_bytes_binary_dictionary           # required
            trace_state: string | string_dictionary 
            parent_span_id: 8_bytes_binary | 8_bytes_binary_dictionary 
            name: string | string_dictionary                              # required
            kind: int32 
            attributes: *attributes
            dropped_attributes_count: uint32 
            events: 
              - time_unix_nano: uint64 
                name: string | string_dictionary 
                attributes: *attributes
                dropped_attributes_count: uint32
            dropped_events_count: uint32 
            links: 
              - trace_id: 16_bytes_binary | 16_bytes_binary_dictionary 
                span_id: 8_bytes_binary | 8_bytes_binary_dictionary
                trace_state: string | string_dictionary 
                attributes: *attributes
                dropped_attributes_count: uint32 
            dropped_links_count: uint32
            status: 
              code: int32 
              status_message: string | string_dictionary
```