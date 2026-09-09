# Conditional composite context entries

Context entries can group request metadata and make that group available only
under explicit conditions. OTLP/gRPC and Kafka propagation policies can select
a specific member of that group. The member is not propagated when its parent
entry is absent, even if the underlying header exists independently.

See the runnable configuration in
[`examples/composite-context.yaml`](../examples/composite-context.yaml).
It receives OTLP/gRPC on `127.0.0.1:4317` and exports to an OTLP/gRPC server
on `127.0.0.1:5317`.

## Capture groups and exact references

A capture rule with `store_as` declares an unconditional composite:

```yaml
headers:
  - match_names: [x-customer, x-workspace]
    store_as: request_identity
  - match_names: [environment]
```

`request_identity` selects the whole group. It can contain either or both
captured fields. Capture does not require every listed header to arrive.
`request_identity:x-workspace` selects only its workspace field.
`environment` selects a standalone, singleton-field entry.

Bare `x-workspace` is not a valid reference in this configuration. Reference
resolution never searches inside arbitrary composites and never falls back
from a qualified member to an independent entry.

Names are normalized to lowercase. The `:` separator is reserved for member
qualification; entry and member names cannot themselves contain `:`.
Unknown entries, unknown members, conflicting visible definitions, and
unqualified composite references in field-only positions are configuration
errors, not runtime absence.

Singleton-field means one field in the schema, not one occurrence in a request.
All occurrences of a repeated header remain available in input order. Empty
values are present values.

## Explicit conditions and atomic presence

```yaml
policies:
  context:
    entries:
      production_workspace:
        - type: transport_header
          name: request_identity:x-customer
          as: customer
        - type: transport_header
          name: request_identity:x-workspace
          as: workspace
        - type: transport_header_match
          name: environment
          value: production
          match: all
```

A member's optional `as` names the field within the new composite; without it,
the selected source field's name is used. This is separate from propagation's
outbound header naming strategy.

This composite is present only when both members exist and the condition
succeeds. Conditions are conjunctive and do not add dimensions to the entry.
Conditional fields are currently primitive transport fields, including
qualified members of capture groups. Derived entries cannot be inputs to
other derived entries in this implementation.

Text comparisons are exact and case-sensitive. A match condition must specify
how repeated values participate:

- `match: any`: at least one value equals the configured text.
- `match: all`: every value equals the configured text.

Both require at least one value. Missing fields never match, and binary values
are not implicitly converted to text.

Definitions accumulate from engine, group, and pipeline policies without
shadowing. Scope remains part of a derived entry's identity. Sibling scopes
may reuse names, but definitions visible to the same pipeline cannot conflict.
Primitive transport entry names are collected engine-wide from capture and
component producer declarations.

## Propagating a conditional member

```yaml
header_propagation:
  default:
    selector:
      type: named
      named: [production_workspace:workspace]
    action: propagate
    name: preserve
```

This rule emits workspace headers only through the conditional composite:

| Input | Output through this rule |
| --- | --- |
| Customer, workspace, and production environment | All workspace values |
| Repeated workspace values | All occurrences, in input order |
| Workspace but no customer | Nothing |
| Development environment | Nothing |
| Both production and development environment values | Nothing with `match: all` |

An independent propagation rule can explicitly select a primitive field.
The qualified rule itself never bypasses its parent's presence condition.

Selection and naming are independent:

| Selection | `name: stored_name` | `name: preserve` |
| --- | --- | --- |
| `request_identity` | Every selected value uses `request_identity` | Original header names |
| `request_identity:x-workspace` | Workspace values use `request_identity` | Original workspace header name |
| `production_workspace:workspace` | Workspace values use `production_workspace` | Original workspace header name |

Both the default `named` list and override `match.stored_names` accept these
references. Overrides retain first-match order; a qualified override does not
implicitly outrank an earlier whole-entry override. An absent conditional
selection does not match. Within a selector list, the first present selection
of an occurrence determines its naming. Overlapping selectors do not duplicate
an occurrence, but distinct repeated input occurrences are preserved.
`all_captured` continues to select primitive transport entries, not every
derived context entry.

## Binding and runtime contracts

Cardinality belongs to the consumer. Header propagation accepts multiple values.
A scalar field binding distinguishes absence from multiple values and reports
an error for the latter; it does not choose the first occurrence.

The compiler resolves entry and field identities, conditions, naming, and
override precedence before runtime construction. Original wire names are
retained only for source fields whose consumers can require them. A qualified
field reference does not itself require storing the original wire-name string.

Contexts share immutable values and indexes across clones. Derived entries
reference primitive values rather than copying their bytes. Bound producer
updates recompute composite presence and preserve copy-on-write isolation.
Unbound raw mutation invalidates compiled access instead of leaving stale
composites available.

Contexts retain their layout. An incompatible layout after live reconfiguration
is an explicit access error, not an opportunity to reinterpret slot numbers.
Exporters reject such messages rather than exporting them without their selected
context. A newly installed rule requiring an original name also rejects a
selected value if the earlier capture deliberately omitted that information.
Retrying the unchanged message against the same incompatible binding
does not repair it; these propagation failures are permanently nacked.

This slice implements transport-based composites and their propagation.
Authorized-identity sources, network sources, nested derived entries, general
N:M projection, and routing/batching/resource-control integrations remain future
work under RFC 0004.
