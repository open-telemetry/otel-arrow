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
# Attributes Arrow Schema (declaration used in other schemas)
attributes: &attributes                                 # arrow type = map
  - key: string_dictionary | string
    value:                                              # arrow type = sparse union
      str: string_dictionary | string
      i64: int64
      f64: float64
      bool: bool
      binary: binary_dictionary | binary
      cbor:  binary                                     # cbor encoded complex attribute values

# Exemplar Arrow Schema (declaration used in other schemas)
exemplars: &exemplars
  - attributes: *attributes
    time_unix_nano: timestamp
    value:                                              # arrow type = sparse union
      i64: int64
      f64: float64
    span_id: 8_bytes_binary_dictionary | 8_bytes_binary
    trace_id: 16_bytes_binary_dictionary | 16_bytes_binary


# Metrics Arrow Schema
# OTLP univariate metrics are represented with the following Arrow Schema.
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
          # This section represents the standard OTLP metrics as defined in OTEL v1 
          # specifications.
          #
          # Named univariate metrics as their representation allow to represent each
          # metric as independent measurement with their own specific timestamps and
          # attributes.
          #
          # Shared attributes and timestamps are optional and only used for optimization
          # purposes.
          univariate_metrics:                             
            - name: string | string_dictionary            # required
              description: string | string_dictionary
              unit: string | string_dictionary 
              shared_attributes: *attributes              # attributes inherited by data points if not defined locally 
              shared_start_time_unix_nano: timestamp      # start time inherited by data points if not defined locally
              shared_time_unix_nano: timestamp            # required if not defined in data points
              data:                                       # arrow type = sparse union
                - gauge: 
                    shared_attributes: *attributes              # attributes inherited by data points if not defined locally 
                    shared_start_time_unix_nano: timestamp      # start time inherited by data points if not defined locally
                    shared_time_unix_nano: timestamp            # required if not defined in data points
                    data_points: 
                      - attributes: *attributes
                        start_time_unix_nano: timestamp
                        time_unix_nano: timestamp         # required if not defined as a shared field in the metric
                        value:                            # arrow type = sparse union
                          i64: int64 
                          f64: float64 
                        exemplars: *exemplars
                        flags: uint32
                  sum:
                    shared_attributes: *attributes              # attributes inherited by data points if not defined locally 
                    shared_start_time_unix_nano: timestamp      # start time inherited by data points if not defined locally
                    shared_time_unix_nano: timestamp            # required if not defined in data points
                    data_points: 
                      - attributes: *attributes
                        start_time_unix_nano: timestamp
                        time_unix_nano: timestamp         # required
                        value:                            # arrow type = sparse union
                          i64: int64
                          f64: float64
                        exemplars: *exemplars
                        flags: uint32
                    aggregation_temporality: int32_dictionary
                    is_monotonic: bool
                  summary:
                    shared_attributes: *attributes              # attributes inherited by data points if not defined locally 
                    shared_start_time_unix_nano: timestamp      # start time inherited by data points if not defined locally
                    shared_time_unix_nano: timestamp            # required if not defined in data points
                    data_points: 
                      - attributes: *attributes
                        start_time_unix_nano: timestamp
                        time_unix_nano: timestamp               # required
                        count: uint64
                        sum: float64
                        quantile: 
                          - quantile: float64
                            value: float64
                        flags: uint32
                  histogram:
                    shared_attributes: *attributes              # attributes inherited by data points if not defined locally 
                    shared_start_time_unix_nano: timestamp      # start time inherited by data points if not defined locally
                    shared_time_unix_nano: timestamp            # required if not defined in data points
                    data_points:
                      - attributes: *attributes
                        start_time_unix_nano: timestamp
                        time_unix_nano: timestamp
                        count: uint64
                        sum: float64
                        bucket_counts: []uint64
                        explicit_bounds: []float64
                        min: float64
                        max: float64
                        exemplars: *exemplars
                        flags: uint32
                    aggregation_temporality: int32_dictionary
                  exp_histogram:
                    shared_attributes: *attributes              # attributes inherited by data points if not defined locally 
                    shared_start_time_unix_nano: timestamp      # start time inherited by data points if not defined locally
                    shared_time_unix_nano: timestamp            # required if not defined in data points
                    data_points:
                      - attributes: *attributes
                        start_time_unix_nano: timestamp
                        time_unix_nano: timestamp
                        count: uint64
                        sum: float64
                        scale: int32
                        zero_count: uint64
                        positive:
                          offset: int32
                          bucket_counts: []uint64
                        negative:
                          offset: int32
                          bucket_counts: []uint64
                        min: float64
                        max: float64
                        exemplars: *exemplars
                        flags: uint32
                    aggregation_temporality: int32_dictionary

# Metrics Arrow Schema
# OTLP multivariate metrics are represented with the following Arrow Schema.
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
        # Native support of multivariate metrics (not yet implemented)
        #
        # Multivariate metrics are related metrics sharing the same context, i.e. the same
        # attributes and timestamps.
        #
        # Each metrics is defined by a name, a set of data points, and optionally a description
        # and a unit.
        multivariate_metrics:                       
            attributes: *attributes                   # All multivariate metrics shared the same attributes
            start_time_unix_nano: timestamp           # All multivariate metrics shared the same timestamps
            time_unix_nano: timestamp                 # required
            metrics:                                  # arrow type = sparse union
              - gauge:
                  name: string | string_dictionary            # required
                  description: string | string_dictionary
                  unit: string | string_dictionary 
                  value:                                    # arrow type = dense union
                    i64: int64 
                    f64: float64
                  exemplars: *exemplars
                  flags: uint32  
                sum:
                  name: string | string_dictionary            # required
                  description: string | string_dictionary
                  unit: string | string_dictionary
                  value:                                    # arrow type = dense union
                    i64: int64
                    f64: float64
                  exemplars: *exemplars
                  flags: uint32
                  aggregation_temporality: int32_dictionary
                  is_monotonic: bool
                summary:
                  name: string | string_dictionary            # required
                  description: string | string_dictionary
                  unit: string | string_dictionary
                  count: uint64 
                  sum: float64
                  quantile: 
                    - quantile: float64
                      value: float64
                  flags: uint32
                histogram:
                  name: string | string_dictionary            # required
                  description: string | string_dictionary
                  unit: string | string_dictionary
                  count: uint64
                  sum: float64
                  bucket_counts: []uint64
                  explicit_bounds: []float64
                  exemplars: *exemplars
                  flags: uint32
                  min: float64
                  max: float64
                  aggregation_temporality: int32_dictionary
                exp_histogram:
                  name: string | string_dictionary            # required
                  description: string | string_dictionary
                  unit: string | string_dictionary
                  count: uint64
                  sum: float64
                  scale: int32
                  zero_count: uint64
                  positive:
                    offset: int32
                    bucket_counts: []uint64
                  negative:
                    offset: int32
                    bucket_counts: []uint64
                  exemplars: *exemplars
                  flags: uint32
                  min: float64
                  max: float64
                  aggregation_temporality: int32_dictionary


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
          - time_unix_nano: timestamp 
            observed_time_unix_nano: timestamp 
            trace_id: 16_bytes_binary | 16_bytes_binary_dictionary 
            span_id: 8_bytes_binary | 8_bytes_binary_dictionary
            severity_number: int32_dictionary 
            severity_text: string | string_dictionary 
            body:                                           # arrow type: sparse union
              str: string | string_dictionary 
              i64: int64 
              f64: float64 
              bool: bool 
              binary: binary | binary_dictionary
              cbor: binary_dictionary | binary              # cbor encoded complex body value
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
          - start_time_unix_nano: timestamp                               # required 
            end_time_unix_nano: timestamp                                 # required
            trace_id: 16_bytes_binary | 16_bytes_binary_dictionary        # required
            span_id: 8_bytes_binary | 8_bytes_binary_dictionary           # required
            trace_state: string | string_dictionary 
            parent_span_id: 8_bytes_binary | 8_bytes_binary_dictionary 
            name: string | string_dictionary                              # required
            kind: int32_dictionary 
            attributes: *attributes
            dropped_attributes_count: uint32 
            events: 
              - time_unix_nano: timestamp 
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
              code: int32_dictionary 
              status_message: string | string_dictionary
```