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

This requirement ensures temporal consistency between related measurements and forms the foundation of OTel-Arrow's multivariate design.

## OTel-Arrow Protocol Design

OTel-Arrow implements multivariate metrics by definition. When
multiple metric data points have identical resource, instrumentation
scope, and timestamp, they are by definition multivariate.

## Phased exploration

### Phase 1

During Phase 1, we explored potential cost savings from the use of
multivariate metrics, because there is a natural compression benefit
from grouping related data.
The implementation used explicit configuration, instead of a natural
multivarate metric API.

For example, a configuration stating that `memory.usage` and
`memory.limit` (in some scope) should be multivariate, then any
observations of those sharing resource and timestamp would become
multivariate in the representation.  However, we removed this feature
from the reference implementation prior to stabilizing the Phase 1
protocol.

### Phase 2

In phase 2, as we implement an end-to-end OTAP pipeline, we will
examine the use of multivariate metrics APIs.

We will explore synthesizing well-defined multivariate metrics APIs
using OTel-Weaver. As an example relevant for self-observability in a
pipeline scenario, consider the recording of `success`, `failed`,
`dropped`, `compressed_bytes`, and `uncompressed_bytes` counts
associated with a single request in the pipeline.  This is a natural
case for multivariate metrics, and we will explore recording these
attributes in the correct types in the "OTAP-direct" SDK.
