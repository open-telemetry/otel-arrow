# OTel-Arrow Multivariate Metrics Design

## Overview

Multivariate metrics in OpenTelemetry represent multiple measurements that are logically related and captured at the same moment in time. These measurements provide different dimensions of the same system property and must be temporally aligned to provide an accurate system state representation.

## OpenTelemetry Requirement

The OpenTelemetry API model defines a critical requirement for multivariate metrics:

```
The API MUST treat observations from a single Callback as logically
taking place at a single instant, such that when recorded,
observations from a single callback MUST be reported with identical
timestamps.
```

This requirement ensures temporal consistency between related measurements and forms the foundation of OTel-Arrow's multivariate implementation.

## OTel-Arrow Protocol Implementation

OTel-Arrow implements multivariate metrics through:

1. **Arrow Record Batch Structure**: The protocol leverages Arrow's columnar format to efficiently store related metrics with identical timestamps.

2. **Batch-oriented Processing**: Multiple metrics from a single callback are processed together, preserving their relationship.

3. **Shared Timestamp Column**: Metrics observed in the same callback share a single timestamp column in the Arrow record batch, guaranteeing timestamp consistency.

## Golang Reference Implementation

The reference implementation in this repository handles multivariate metrics by:

### Producer Side

- The metrics producer captures all observations from a single callback.
- A single timestamp is generated and applied to all metrics within the callback.
- When encoding to Arrow format, the implementation ensures these observations share identical timestamp fields.
- The record batch structure preserves this temporal relationship during serialization.

### Consumer Side

- When processing Arrow record batches, the consumer respects the shared timestamps.
- Metrics with identical timestamps are recognized as having occurred simultaneously.
- The implementation maintains these relationships throughout the processing pipeline.

## Benefits

This approach to multivariate metrics provides:

- **Correctness**: Ensures OpenTelemetry's requirement for identical timestamps is met.
- **Efficiency**: Reduces storage overhead by sharing timestamp information across related metrics.
- **Analytical Integrity**: Preserves critical temporal relationships for accurate system analysis.

## Example

When a callback produces multiple measurements (e.g., CPU usage percentage and memory usage), OTel-Arrow ensures these measurements share a single timestamp in the resulting Arrow record batch, correctly representing that these observations occurred simultaneously.
