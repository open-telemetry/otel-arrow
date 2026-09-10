# otel-arrow-dfe-admin-types

This crate is currently pre-1.0. Its public API may evolve between minor
releases.

## Overview

`otel-arrow-dfe-admin-types` contains the shared request, response, query, and
model types used by the OTAP Dataflow admin server and the public admin SDK.

## Key Types

- `operations`: wait and delete options plus typed operation errors
- `engine`: engine reconciliation, deletion, and probe status models
- `groups`: pipeline-group status and shutdown responses
- `pipelines`: rollout, reconfiguration, shutdown, and condition models
- `telemetry`: metrics and live-log query and response models

This is an internal implementation crate shared by the admin server and public
SDK. External integrators should depend on `otel-arrow-dfe-admin-api`, which
re-exports these model modules as part of its public surface.
