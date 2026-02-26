# URN Format for OTAP Dataflow (config=model-v1)

This document defines the canonical URN format for node identifiers in OTAP
dataflow configuration files. All nodes MUST use the format below.

## Canonical format

Full form (always valid):

```xml
urn:<namespace>:<kind>:<id>
```

Where:

- `<namespace>` is a vendor or vendor+product identifier (e.g., `otel`, `microsoft_azure`).
- `otel` is reserved for OpenTelemetry-provided nodes.
- `<id>` identifies the node type.
- `<kind>` is one of: `receiver`, `processor`, `exporter`.

Note: the namespace reflects ownership/standardization of the node type, not the
Rust crate or module that implements it. OpenTelemetry-provided nodes use the
`otel` namespace even if their implementation lives in the `otap` crate.

## OTel shortcut form

For OpenTelemetry nodes only, a shortcut form is permitted:

```xml
<kind>:<id>
```

This is expanded internally to:

```xml
urn:otel:<kind>:<id>
```

The shortcut form MUST NOT be used for any namespace other than `otel`.
Identifiers starting with `urn:` are always treated as fully-qualified URNs.

## Identifier rules

- `:` is a reserved separator and MUST NOT appear inside any component.
- Components are treated as opaque identifiers, but for OTAP dataflow we restrict
  components to `[a-z0-9._-]` for unambiguous parsing.
- Empty components are not allowed.

## Examples

Full form examples:

```text
urn:otel:receiver:otlp
urn:otel:processor:debug
urn:otel:exporter:otap
urn:microsoft_azure:exporter:monitor
```

Shortcut form (OTel only):

```text
receiver:otlp
processor:debug
```
