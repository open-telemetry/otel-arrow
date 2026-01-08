# Security and privacy guide

Status: Draft

This document defines security and privacy constraints for internal telemetry.

Telemetry is a high leverage system: it can accidentally become a data
exfiltration path, leak secrets, or expose sensitive topology. These rules are
designed to prevent that.

## Principles

- Telemetry MUST NOT include secrets, credentials, or sensitive personal data.
- Telemetry SHOULD include only what is required for operations and debugging.
- Telemetry MUST be safe under failure and safe under adversarial inputs.
- Access to telemetry SHOULD follow least privilege.

## Data that must never appear in telemetry

The following MUST NOT be recorded in metrics, events, traces, or attributes:

- credentials, API keys, bearer tokens, cookies, session secrets
- private keys, certificates, signing material
- raw customer payloads or unredacted message bodies
- email addresses, phone numbers, full names, physical addresses
- full IP addresses if they can identify individuals (normalize or bucket
  instead)
- raw URLs containing query parameters or user-provided path segments
- raw SQL queries, raw stack traces in high volume contexts (see Exceptions)

If in doubt, do not emit it.

Important note: The system must allow users, if they choose to do so, to log
certain sensitive data (e.g. `user_id`) only when it is gated behind an explicit
debug mode.

## Allowed data with constraints

The following are generally allowed if they remain bounded and non-sensitive:

- stable internal IDs for entities (pipelines, nodes, channels), as defined in
  the entity model
- categorical outcomes and reasons (closed enums), for example drop reasons
- normalized forms of user input, for example route templates instead of raw
  paths
- bounded numeric values describing system state and performance

## Normalization and redaction

When context is useful but high cardinality or sensitive, normalize:

- URL path -> route template
- SQL -> normalized fingerprint
- IP address -> prefix or bucket
- error message -> error class or error type

Do not emit raw content that can include secrets or user identifiers.

## Exceptions and stack traces

Exceptions often include sensitive data. Rules:

- Use structured exception attributes (e.g. `exception.type`) when needed.
- `exeption.message` MUST NOT include sensitive data or raw user input.
- `exception.stacktrace` SHOULD be gated behind:
    - debug severity, or
    - an explicit configuration flag
- Stack traces MUST NOT be emitted on hot paths by default.

## Events, body size, and spill risk

Events are exported as logs.

- Prefer small, queryable fields in attributes.
- Large payloads SHOULD go into the event body only when strictly required.
- Do not emit unbounded or repetitive bodies at high volume.

Recommended practice:

- Keep event bodies small and bounded.
- When details are required, emit a stable error type in attributes and keep the
  long detail behind debug-level gating.

## Trace correlation

When exporting events as logs and trace context exists:

- include trace correlation (trace id and span id) so operators can pivot
- do not copy trace ids into custom attributes unless required by tooling

Trace and Span ids are not secrets but they can be used to join information
across
systems. Treat them as internal identifiers.

## Schema endpoint security

If the system exposes a runtime endpoint that returns the current signals, or
resolved schema:

- They SHOULD be protected by authentication and authorization, or limited to
  trusted network boundaries.
- They MUST be configurable to disable access entirely.
- They MUST implement rate limiting to prevent abuse.
- They MUST NOT expose secrets or raw configuration values.
- Treat the endpoint as sensitive because it can reveal topology and
  identifiers.

## Metrics and diagnostic endpoints (/metrics, /status)

If the system exposes metrics scrape endpoints (for example Prometheus-style) or
diagnostic endpoints:

- They SHOULD be protected by authentication and authorization, or limited to
  trusted network boundaries.
- They MUST NOT expose secrets or raw configuration values.
- They SHOULD be designed to avoid unbounded responses (for example unbounded
  label sets or dumping full topology on every request).
- If an endpoint includes topology identifiers (pipelines, nodes, channels),
  treat it as sensitive.

## Data retention

- Data retention SHOULD be appropriate for the sensitivity of the data.
- If telemetry can include customer-adjacent signals, apply stricter retention
  and access constraints.

## Review checklist

For any telemetry addition or change:

- No secrets or personal data is recorded.
- Attributes are bounded and normalized where appropriate.
- Stack traces are gated and not emitted by default on hot paths.
- Schema endpoint exposure is safe for the target deployment.
- Documentation includes any special handling or risk notes.
