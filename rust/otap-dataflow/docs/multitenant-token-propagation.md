# Multitenant Tenant Tokens and Cross-Boundary Propagation

Status: prototype. This document describes the design and implementation of
the tenant token prototype in this workspace, following the multitenancy
design in [open-telemetry/otel-arrow#3583][pr].

[pr]: https://github.com/open-telemetry/otel-arrow/pull/3583

## Problem

A collector that serves many tenants has to answer three questions about every
request it accepts:

1. **Who is this?** Something in the request -- a header, the peer address, the
   pipeline it landed in -- identifies a tenant.
2. **Where does it go?** That identity selects a destination: a topic, a
   partition, a backend.
3. **What travels with it?** Some of that identity must reach the backend, and
   the backend's spelling of it is rarely the client's spelling.

The pre-existing transport header policy (see
[transport-headers.md](transport-headers.md)) answers question 3 alone, by
capturing named headers and forwarding a filtered subset. It cannot answer 1 or
2, because it has no notion of an identity distinct from a wire name, and it
pays for what it captures: an owned `String` name and an owned `Vec<u8>` value
per header per request.

Tenant tokens answer all three with one declaration, and cost one allocation
per request in total.

## Core concepts

### A token is a portable identity, not a header

A **tenant token** is a named set of **extractors**, each of which resolves one
**key** from the request:

```yaml
tenant_tokens:
  edge_tenant:
    extractors:
      - key: customer_id
        transport_header: x-tenant-id
        retain: true
```

A token resolves **all or nothing**: `edge_tenant` exists for a request only if
every one of its extractors resolved. This is what makes a token a usable
identity -- downstream code tests "is this an `edge_tenant`?" rather than
reasoning about partially-populated header maps. Put independently optional
inputs in separate tokens.

Four extractor kinds exist:

| Kind | Source |
| --- | --- |
| `transport_header: <name>` | An inbound header, matched case-insensitively |
| `generic_key: <value>` | A static value, minting a local identity |
| `remote_address: true` | The peer socket address |
| `imported_key: <name>` | A value handed across a pipeline boundary |

`retain: false` (the default) is **match-only**: the value participates in
conditions but is never stored, so it costs zero bytes per request.
`retain: true` gives the key a value slot, so an exporter can emit it and a
boundary policy can offer it downstream. This flag is the one thing that
distinguishes keys in the packed layout, and it is the obvious candidate for
inference: a liveness pass over the pipeline graph knows which keys a reachable
consumer actually propagates, so `retain` could be derived rather than
declared.

**A token never names an egress header.** This is the central design decision.
A wire name is a property of the backend being written to, not of the tenant,
so the same key is `x-scope-orgid` for one backend and `x-acme-customer` for
another. Naming therefore lives on the node that touches the wire:

```yaml
otlp_exporter:
  type: exporter:otlp_grpc
  config:
    tenant_headers:
      - { key: customer_id, header: x-scope-orgid }
```

An earlier iteration put `propagate_as: x-scope-orgid` on the extractor. It was
removed: it forced a site-specific decision into a definition that is shared by
the whole engine, and made a token untranslatable when two exporters disagreed.

### Conditions are first-match-wins

Consumers declare **conditions** over token keys. All entries of a condition
must match; an entry with no `value` is a wildcard requiring only presence:

```yaml
routes:
  - entries: [{ key: customer_id, value: tenant-a }]
    topic: tenant_a
  - entries: [{ key: customer_id }]     # any resolved customer_id
    topic: tenant_shared
default_topic: unmatched
```

## Implementation: the hash join

Conditions are compiled, not interpreted. The design is the Envoy-style
compile-time hash join: all string comparison happens once at startup, and each
request costs a fixed number of hash lookups regardless of how many conditions
are declared.

The registry lives in `crates/config/src/tenant/compiled.rs` and is built once
per pipeline group by the controller
(`Controller::build_tenant_token_registry`), which walks every node config in
the group collecting declared conditions before any node starts. It is shared
through `PipelineContext::tenant_token_registry()`.

### Build phase (startup)

- Key names are interned to `KeyId(u16)`; token names to `TokenIdx(u16)`.
- Each token gets a `key_mask: u64` -- one bit per extractor.
- Header extractors go into `header_index: name -> SmallVec<[Slot; 2]>`, keyed
  by the lowercased header name. Static extractors (`generic_key`,
  `remote_address`) go into a flat `Vec`. Imported extractors go into
  `import_index`, keyed by the upstream key name.
- Conditions are grouped by **signature**: the pair of (sorted fixed keys,
  sorted wildcard keys) they test. Conditions sharing a signature share a hash
  table, because they hash the same terms in the same order.
- A signature applies to a token only if `required_keys` is a subset of the
  token's keys. Each applicable `(token, signature)` pair is assigned a dense
  `PairSlot`.

### Resolve phase (per request)

1. Reset a reusable, receiver-owned `TokenScratch`. Steady state allocates
   nothing here.
2. Run static extractors, then make **one pass over the request headers**,
   clearing `key_mask` bits as extractors are satisfied.
3. A token whose word reached `0` is resolved. If none resolved, the request
   carries no tenant context at all and the whole feature costs one branch.
4. Resolve each key's value to a **symbol** by exact dictionary lookup, then
   pack one word per allocated `PairSlot`, projecting the symbols onto the
   signature's `fixed_keys` **in signature order**. Build and probe share one
   `pack_symbols` helper and one `SignatureLayout`, so the two cannot drift.

Token resolution being all-or-nothing yields a useful simplification: if a
signature applies to a resolved token, every wildcard key it requires is
necessarily present, so no per-request wildcard presence mask is needed.

### Probe phase (per condition evaluation)

`ConditionSet::first_match` is one bit test plus one integer lookup per bound
`(token, signature)` pair, keeping the lowest matching `ConditionIdx` so
first-match-wins is preserved across signatures. It allocates nothing.

The integer it looks up is a packed tuple of dictionary symbols, not a digest,
so the lookup decides equality exactly -- see
[Matching is exact](#matching-is-exact-no-collision-can-cross-wires).

### The packed request context

The per-request result is a single `Arc<[u64]>` stored in
`Context.request`. `Arc<[u64]>` rather than `Arc<[u8]>` because the latter
carries no 8-byte alignment guarantee; bytes are read back with
`bytemuck::cast_slice`, so no `unsafe` is involved.

Nothing in the layout is self-describing. Every key is registered, so a value
is addressed by the **value slot** its key occupies in the registry, and no
name, key id or descriptor travels with it:

```text
word 0     : n_fp:16 | n_slots:16 | epoch:16 | bag_len:16
word 1     : resolved token bitmask
words 2..  : n_fp packed symbol words, indexed by PairSlot
then       : ceil(n_slots/2) words holding n_slots u32 offsets, indexed by
             registry value slot; each addresses a length-delimited AnyValue,
             or is EMPTY_OFFSET
then       : the byte blob, zero padded to a word boundary
```

Values are stored as OTLP. A slot holds a single `u32` offset rather than an
offset and a length, because the encoding is length-delimited and `AnyValue`
carries its own type -- so the two fields the slot used to spend on `len` and
`kind` are already in the bytes. The blob needs no stored length either: every
entry delimits itself, so the trailing zero padding is simply never addressed.

A key gets a value slot only if some compiled consumer propagates its value.
The rest are **match-only**: they are decided entirely by their contribution to
a packed symbol word, so their bytes are never copied into a context at all.
Their values are still matched exactly -- the comparison happens at the
receiver against the registry's copy of the literal -- so this costs no
strictness. That is the only distinction the layout draws between keys, and it
is a build-time property, not a per-entry flag.

Reading a key's value is therefore an array index --
`registry.value_slot(key)` then `view.slot_value(slot)` -- rather than a scan
looking for a matching name or key id.

The blob opens with the **bag**: `bag_len` bytes for the keys whose names are
demanded, each written as `<tag> <len> <KeyValue>` -- already tagged, so the
region is a complete OTLP repeated field rather than an input to an encoder.
Value-only entries follow as bare `<len> <AnyValue>`. Both forms end in a
length-delimited `AnyValue`, and a slot points at that length prefix in either
case, so `slot_value` reads them identically and does not care which region a
key landed in.

There is no name-bearing region for captured headers. Every name in the
context comes from configuration, never from the wire -- see
[Retiring the capture policy](#retiring-the-capture-policy).

**Contexts are only meaningful to registries that agree on the layout.** A
registry is built per pipeline group, but topics are declared engine-wide and
are visible to all groups, so a topic hop can join *two different registries*.
What makes that safe is not shared construction, it is agreement: token
definitions are engine-wide and interned in sorted order before any
group-specific work, so every group derives the same key ids and the same
value slots. Conditions differ per group and pair slots with them, which does
not matter across a boundary because symbol words are not exported.

That agreement is an invariant, so it is stated rather than assumed. `epoch`
is the deployment generation mixed with a digest of the ordered value-slot
layout, and `resolve_imported` drops an upstream context whose epoch differs
instead of reading slots as something they are not. This matters most for a
change the design still wants: the baseline allows tokens to be declared per
group and per pipeline, which would make layouts genuinely diverge. The digest
turns that from a silent misread into a dropped context.

A hop that leaves the registry entirely -- persistence across a restart, or
another engine -- has to re-resolve from transport headers instead.

This field **replaces** `Context.transport_headers` rather than adding to it,
which is why the feature is net-negative on memory. The `request_context`
benchmark measures where that cost actually sits, and the answer is narrower
than it first appears:

Allocations and bytes per request, base = baseline, tok = tenant token:

| phase | matched | base allocs | tok allocs | base bytes | tok bytes |
| --- | --- | --- | --- | --- | --- |
| capture | 1 | 5 | 1 | 388 | 56 |
| capture | 2 | 8 | 1 | 421 | 72 |
| capture | 4 | 14 | 1 | 546 | 160 |
| carry | any | 0 | 0 | 0 | 0 |
| propagate | any | 0 | 0 | 0 | 0 |
| attributes | any | n/a | 0 | n/a | 0 |

Capture cost `2 + 3n` -- an `Arc`, a `Vec`, and a name, wire name and value
per header. It is now **1**, flat, and the bytes fall with it because a value
slot spends no name and no descriptor.

Carry and propagation were already free: `TransportHeaders` is an
`Arc<Vec<TransportHeader>>`, so a hop was an `Arc` bump and propagation
borrowed. The win is per request, not per hop, and claiming otherwise would
overstate it.

Wall time, same run, mean nanoseconds:

| phase | matched | baseline | tenant |
| --- | --- | --- | --- |
| capture / resolve | 1 | 126 | 77 |
| capture / resolve | 2 | 229 | 115 |
| capture / resolve | 4 | 485 | 149 |
| carry (per hop) | any | 10 | 13 |
| propagate | 4 | 14 | 16 |
| end to end | 1 | 157 | 104 |
| end to end | 2 | 244 | 158 |
| end to end | 4 | 552 | 207 |
| attributes | 4 | n/a | 5 |

Resolution beats capture at every width, by 39% at one key and 69% at four,
and end to end is 34% to 63% faster. Two things pay for that. Registered
header names are screened by a one-word Bloom filter over length and first
byte before anything is hashed, so the four headers a pipeline does not want
-- content type, user agent, encoding, timeouts -- are rejected without a map
lookup. And since gRPC and HTTP/2 already require lowercase names, case
folding is skipped unless a name actually needs it.

Two costs went **up**, and both are worth stating. A hop is 3ns dearer,
because `Arc<[u64]>` is a fat pointer where `Arc<Vec<_>>` was thin; against
roughly 300ns saved per request at four keys, that pays for itself until
about a hundred hops. Reading one value is about 2ns dearer, because a slot
addresses an encoded `AnyValue` and the read skips a tag and two varints
rather than indexing a struct field.

`attributes` has no baseline row because the baseline cannot do it: emitting
the request's context as telemetry attributes would mean encoding a
`KeyValue` per header. Here it is a borrow of an already formed repeated
field, and the 5ns is the memcpy.

Limits: `MAX_TOKENS = 64`, `MAX_TOKEN_KEYS = 64`, `MAX_VALUE_BYTES = 65535`.

### Matching is exact: no collision can cross wires

**A tenant condition matches only when the request's values are equal, byte
for byte, to the values the operator declared.** No hash comparison decides
routing. This is the property everything else in this section is arranged to
protect, because the failure it prevents is one tenant's data being delivered
to another tenant's pipeline.

An earlier version of this design did not have that property, and it is
recorded here because the reasoning that removed it is the reasoning that
should keep it out. Conditions were compiled to a 64-bit `xxh3` fingerprint of
their literals, resolution fingerprinted the request the same way, and
`first_match` looked the fingerprint up in a map and stopped. Values were never
re-checked. For match-only keys nothing was carried, so downstream they *could
not* be re-checked -- the evidence was gone by construction.

That is a hash join with the verification step deleted. A database does not do
this. A hash join uses the hash to choose a bucket and then compares the actual
join keys, because the hash is an index, never the predicate. Skipping the
comparison converts a 64-bit coincidence into a routing decision.

The odds argument is not good enough here. `xxh3` is not collision resistant --
it is not meant to be -- so an attacker who can set a header does not wait for
a coincidence, they search for one offline against a public function and then
replay it forever. A boundary that separates tenants cannot rest on that.

#### Symbols, not digests

The fix is the same move a database makes: dictionary-encode the join keys.

Every literal any condition can test is known at build time, since conditions
are declared to the registry before it is frozen. Each key therefore gets a
dictionary from literal to a small integer **symbol**, with two reserved
values:

| symbol | meaning |
| --- | --- |
| 0 | the request did not carry this key |
| 1 | the request carried a value no condition declares |
| 2.. | one declared literal each |

Resolution looks each value up in its key's dictionary. **The dictionary owns
the literal bytes and compares them**, so a lookup that lands in the same
bucket by hash still fails on the byte comparison. A value that is not found
takes symbol 1.

A signature then packs its keys' symbols into one word, giving each key
exactly as many bits as its declared literal count needs. That packing is
**injective**: distinct symbol tuples cannot produce the same word. So
comparing two words is comparing two symbol tuples, and comparing two symbol
tuples is comparing two sets of verified values.

The hot path is unchanged in shape -- `first_match` is still one integer
lookup per pair, and the packed context still carries one word per pair. What
changed is that the word is now a dictionary encoding rather than a digest, so
the lookup is exact instead of probabilistic.

Two things fail closed:

- A condition testing a literal the registry never interned is **rejected at
  build time**. Such a literal would otherwise take symbol 1, which every
  unrecognized request value also takes, and the condition would match all of
  them at once. This is the one dangerous case the design creates, so it is an
  error rather than a warning.
- A signature whose symbols do not fit in one word is rejected at build time
  rather than narrowed, since narrowing is exactly the collision this section
  exists to prevent. The budget is generous: four keys with 65,000 declared
  values each, or sixteen keys with sixteen values each.

#### Why match-only keys still cost zero bytes

The property that worried the reader -- `retain: false` carries no bytes, so
how can anything be verified? -- survives, because **verification happens where
the value already is**.

The receiver holds the header value in hand at resolve time. That is where the
dictionary lookup happens, and what it compares against is the registry's own
copy of the operator's literal, which exists once at build time rather than
once per request. Downstream nodes never need the value because they are not
deciding equality; that decision was already made, exactly, and its result is
the symbol.

So a match-only key costs zero per-request bytes *and* is matched exactly.
Those were never in tension; the earlier design just gave up the second one
without needing to.

A symbol is also less revealing than a fingerprint was. A fingerprint is a
digest of the tenant's actual value, and an attacker who observes one can
search for the input. A symbol is an index into the operator's own
configuration, which says only *which* declared value matched.

## Boundaries

A topic is a pipeline boundary, and boundaries are the only place tenant
material can leak between tenants. The design therefore **fails closed**: with
no policy configured, nothing crosses, and both ends of every boundary carry an
explicit allowlist. Belt and braces is deliberate -- removing either end stops
propagation, so no single misconfiguration silently opens the boundary.

### One key, one value

Conditions and values need opposite guarantees, and the baseline design only
addresses the first.

For **conditions**, many-to-many is the point. Every resolved token is probed
against every applicable condition, and a request matching several is normal;
the cross-product is what `PairSlot` enumerates and what first-match-wins
disambiguates.

For **values**, many-to-one is a defect. An exporter asking for `customer_id`
must get one answer, and it must be the same answer on every request that
looks the same.

Two things had to change to guarantee that.

#### A key has one source per resolve path

Staging is per key: `scratch.values` is indexed by `KeyId`, so every extractor
writing a given key writes the same cell. Two tokens binding one key to
different sources therefore race, and the winner depends on the order headers
arrived in the request.

This is not only a propagation bug. Fingerprints are computed per
`(token, signature)` pair, but each one reads the shared per-key cell, so token
A's condition can be evaluated against a value that token B's extractor stored.
The cross-product is honoured at symbol time and violated at staging time.

The build now rejects it:

```text
tenant token 'edge_tenant' binds key 'customer_id' to transport_header
'x-tenant-id' on the request path, but it is already bound there to
transport_header 'x-legacy-tenant'; a key resolves from one source per path,
so give these different key names
```

The rule is per **resolve path**, not registry-wide, because there are two
disjoint ones: `resolve` runs transport-header and static extractors,
`resolve_imported` runs imported-key and static extractors. A key read from a
header on the ingress side and imported on the far side of a boundary is
consistent -- that is precisely how a portable key crosses a boundary keeping
its name -- while two headers feeding one key on the same path is not.

Repeating a key across tokens is still fine and still common, as long as the
source agrees. Tokens that differ in their other keys routinely share an
identity key.

The rule also lands where the earlier discussion already pointed. Wanting
`customer_id` from auth *and* from a header is not one key with two sources;
it is a verified key and a claimed key, which is why that example names them
separately.

#### A value travels only with its evidence

The second defect was subtler. Value slots were filled from staging
unconditionally, so a key retained by a token that never resolved still had
its value packed and emitted, as long as some *other* token resolved.

That inverts the security posture. A token is a conjunction: retaining
`region_id` in a token that also requires `gate_id` says the value may be
carried *when the gate was proven*. Releasing it regardless discards the
evidence and keeps the payload.

Each value slot now carries the mask of tokens that declared its key with
`retain: true`, and the slot is populated only if one of them resolved:

```rust
scratch.slots[slot] = if resolved & self.retain_mask[slot] == 0 {
    ValueRef::default()
} else {
    scratch.values[usize::from(*key)]
};
```

It costs one word per retained key and one AND per pack. Note the mask holds
only tokens that asked to retain: a token that matches on a key without
retaining it does not release the value, so requesting a value and merely
testing it stay distinct.

### Egress: `exporter:topic`

The topic router (`exporter:topic`) is both router and egress boundary:

```yaml
topic_router:
  type: exporter:topic
  config:
    routing:
      tenant_tokens: [edge_tenant]
      routes:
        - entries: [{ key: customer_id, value: tenant-a }]
          topic: tenant_a
      default_topic: unmatched
      export:
        allow_keys: [customer_id]
```

`select_route` probes the condition set; `apply_egress_policy` then calls
`TenantTokenRegistry::export_boundary`, which repacks **only** the allowed keys
into a fresh buffer, keeping each value in its own registry slot. Fingerprints,
the resolved-token mask, the captured headers and every unlisted key are
dropped. The blob is rebuilt rather than masked in place, since the packed
buffer is shared and any byte left in it would stay readable. Without
`export`, the context is set to `None` outright.

This replaced the previous behavior, where `clone_without_context` in the topic
receiver carried the full request context across the hop unconditionally.

### Ingress: `receiver:topic`

```yaml
from_topic:
  type: receiver:topic
  config:
    topic: tenant_a
    tenant_context:
      import:
        allow_keys: [customer_id]
      tokens: [routed_tenant]
```

The inbound context is treated as **evidence, not identity**. It is never
adopted. `BoundaryFilter::admits` screens by key id -- the two registries
agree on key ids, so this is a flag lookup, not a name match -- then
`resolve_imported` re-runs the resolve phase over the admitted values plus this
pipeline's static extractors, producing a context whose tokens and symbol words
belong to the local pipeline. A dedicated tenant pipeline can therefore combine
an imported value with a `generic_key` identity of its own in a single token.

## Batching

`processor:batch` merges requests, so it is the one node that can silently
attribute one tenant's data to another. Two mechanisms address this.

**Partitioning.** `batch_by` gives each condition its own complete set of
per-signal buffers, plus a trailing catch-all partition for data matching
nothing:

```yaml
batcher:
  type: processor:batch
  config:
    batch_by:
      tenant_tokens: [routed_tenant]
      partitions:
        - entries: [{ key: customer_id, value: tenant-a }]
```

Since buffers are per-partition, an output batch belongs to exactly one
partition by construction. Two pieces of per-buffer state had to become
partition-aware: wakeup slots are offset by
`partition * SLOTS_PER_PARTITION`, and the outbound slot key in the ack/nack
calldata is stamped with its partition, since a `SlotKey` is only meaningful
inside its own partition's slot state.

**Merge safety net.** Each `BatchPortion` retains its input's packed context,
and `RequestContextMerger` folds them the way `PeerAddrMerger` folds peer
addresses: the merged batch keeps a context only if every contributing input
agreed, and drops it otherwise. Partitioning normally guarantees agreement, so
this exists to make a disagreement fail closed -- the batch emits no tenant
headers -- rather than mislabel data.

Before this work, the batch processor built every output with
`Context::default()`, so batching destroyed the tenant context entirely.

## Worked example

`configs/engine-conf/tenant_boundary_propagation.yaml` wires the full path:

```text
OTLP receiver -> resource enricher -> topic router (routes by token)
   =>  topic  =>
topic receiver (new context) -> batch (partitioned) -> OTLP exporter
```

The ingress token reads `x-tenant-id` into `customer_id` and retains it. The
router selects a per-tenant topic and exports `customer_id`. Each tenant
pipeline imports `customer_id`, mints its own `routed_tenant` token from it via
`imported_key`, partitions batches by it, and names it for its own backend.

Verified live against two catch-all gRPC servers:

| Inbound | Backend | Outbound |
| --- | --- | --- |
| `x-tenant-id: tenant-a` | A | `x-scope-orgid: tenant-a` |
| `x-tenant-id: tenant-b` | B | `x-acme-customer: tenant-b` |
| `x-tenant-id: zzz` | -- | routed to the unmatched sink |

Two further demos exist:
`configs/engine-conf/topic_multitenant_token_routing.yaml` (routing only) and
`configs/engine-conf/tenant_header_mapping.yaml` (single-pipeline
ingress-to-egress header mapping).

## The context as OTLP attributes

The packed context is a key-value bag with a hand-rolled encoding. The
`self_tracing` encoder already solved the same problem for log attributes by
half-encoding straight to OTLP bytes at the callsite, and the same move applies
here. The payoff is not the encoding itself, it is that the bytes become
*directly appendable* to telemetry: instrumenting a request with its tenant
context stops being a re-encode and becomes a copy.

### The slot becomes one offset

A slot was `value_off:16 | value_len:16 | kind:8`. A slot is instead
a single `u32` offset pointing at the length varint of a `KeyValue.value`
field, so the encoding carries its own size and `AnyValue` carries its own
type. `ValueKind` disappears -- it is a hand-rolled parallel to `AnyValue`, and
a worse one, since `AnyValue` covers int, double, bool and bytes.

```text
value-only key : <vlen> <AnyValue>
bag key        : 0A <klen> <key> 12 <vlen> <AnyValue>
                                     ^-- the slot points here
```

Both are read the same way: take the varint at the slot, slice that many bytes,
and hold an `AnyValue`. The bag form simply has a name in front of it, so one
region of bytes serves both the slot read and the bulk copy with no
duplication.

The size cost is close to nothing. `tenant-a` is 8 raw bytes today plus a
5-byte slot descriptor; as a bare value it is 11 bytes plus a 4-byte slot. The
bag form is the one that costs, roughly doubling, and only keys a bag consumer
actually reaches pay it.

### Names become demand-driven, like values

`retain` already makes value bytes conditional on some compiled consumer
wanting them. Extending the same liveness rule to names gives three levels per
key:

| demand | stored |
| --- | --- |
| match-only | nothing; the symbol decides |
| a consumer propagates the value | `<vlen> <AnyValue>` |
| a consumer reads the whole bag | `0A <klen> <key> 12 <vlen> <AnyValue>` |

This keeps the security posture intact. A name still never travels because it
was on the wire; it travels because a compiled consumer in this deployment
demanded the bag, which is a declaration like any other. The default remains
that nothing propagates.

Bag-level keys must be contiguous, so the layout grows a bag region whose
entries are adjacent and whose slots point inside it.

### The consumer bakes in the field number

OTLP does not use a single field number for attributes:

| destination | field |
| --- | --- |
| `Resource.attributes` | 1 |
| `InstrumentationScope.attributes` | 3 |
| `LogRecord.attributes` | 6 |
| `Exemplar.filtered_attributes` | 7 |
| `Span.attributes` | 9 |

An earlier draft stored the run **untagged** and had the consumer supply the
field number per call, pushing a tag byte before each entry. That is still a
per-entry walk, and it means the stored bytes are an *input to an encoder*
rather than a payload.

The consumer is also the initializer. The SDK sets the pipeline up and later
reads the bytes back, so the destination is known at build time, and the tag
can simply be encoded in place:

```rust
let mut builder = TenantTokenRegistryBuilder::new()
    .with_bag_field(attribute_field::SCOPE);
```

```rust
/// The bagged keys, as a complete OTLP repeated `KeyValue` field.
pub fn attributes(&self) -> &'a [u8]
```

Reading is then a borrow, and writing is one `extend_from_slice` of the whole
region. Nothing is walked per entry, nothing is re-tagged, nothing is
allocated. A different destination is a different registry, not a rewrite --
which is the honest shape, since the pipeline that produced the context and the
SDK that consumes it are configured together.

### Scope attributes amortize the copy

The expected caller passes `INSTRUMENTATION_SCOPE_ATTRIBUTES`, and that choice
does more than pick a number.

Scope attributes are shared by every record under a `ScopeLogs`, `ScopeSpans`
or `ScopeMetrics`, so the tenant context is copied once per scope rather than
once per record. It composes with work the prototype already does: batching is
partitioned by tenant conditions, so an output batch belongs to exactly one
partition and therefore to exactly one tenant context. One batch, one scope,
one copy -- and the more records a batch holds, the better the amortization.

It is also the honest modelling. The tenant context describes the conditions
under which the pipeline produced this telemetry, not a property of each
individual record, which is what an instrumentation scope is for.

### Symbols are resolved from the raw bytes

An earlier draft proposed matching on the encoded `AnyValue` rather than the
raw value, back when matching was a hash comparison. Both halves of that idea
are now gone -- matching is a dictionary lookup, and the lookup happens on raw
bytes -- but the ordering argument that decided it still holds and is worth
keeping.

Resolve stages raw bytes and resolves symbols immediately; pack runs
afterwards and encodes only the values that survived gating. Matching on the
encoded form would invert that, forcing every condition-participating key to
be encoded on the hot path -- including match-only keys, whose entire point is
that their bytes never travel.

The gain it offered was distinguishing the integer `5` from the string `"5"`.
That distinction is empty here: condition literals come from YAML as strings
and are compared against transport header bytes, so no term is ever an
integer.

The cost of resolving on raw bytes is one decode in `resolve_imported` -- skip
a tag byte and two varints -- on the boundary path only, which is not the hot
path.

### Demand is declared per extractor

An extractor carries `bag: true` alongside `retain: true`. `retain` says the
value travels; `bag` says the *name* travels with it, which is what makes the
entry appendable as an attribute. `bag` implies `retain`, since a name with no
value is not an attribute.

This keeps the name in configuration, where full token binding requires it to
be. The key name encoded into the blob is the registry key name, not anything
read off the wire.

### The wildcard extractor

A bag is only useful if something can fill it, and full token binding
deliberately has no way to capture an undeclared key. A wildcard extractor
would reintroduce that, and it has to be justified rather than assumed. It is
**not implemented**; `bag: true` on a declared extractor is.

The distinction that makes it acceptable is that a wildcard is **declared**.
The rejected `capture:` plus `preserve` combination let an inbound request
decide what left the process; a wildcard extractor is an operator writing down
"collect the rest here", and everything downstream still applies -- the
boundary allowlists screen it as a unit, and egress still requires an exporter
to name what it emits.

The blast radius also depends on who consumes it, and the two consumers are not
alike. Feeding a bag into span attributes or self-telemetry keeps it inside the
process, which is the case that motivates the feature. Feeding it back onto the
wire is the case that gave up the enumeration guarantee, and it is the one that
should have to say so.

## Retiring the capture policy

This section is a **migration plan**. The name-bearing region of the packed
layout is already gone; what remains is to move every component still calling
`set_transport_headers` / `transport_headers()` onto the compiled context.

The principle that removed the region is the one that governs the migration:
**a name that travels is a name nobody declared**.

The security posture is the argument. Under `capture:`, a receiver matches
`match_names` and stores whatever it found, wire name included, and an exporter
with `name: preserve` re-emits it. Nobody wrote down that the header would
leave the process. Any header a client can set is one an operator never
reviewed, so the set of names crossing an egress boundary is decided by the
inbound request rather than by configuration.

Under full token binding, a name exists at exactly two places, both declared:
an extractor that reads it and an exporter that writes it. In between there is
a key id and a value slot. There is no *as captured* name strategy, because
there is no captured name to preserve -- the ingress name was compiled out at
build time and the egress name is created fresh from the exporter's own map.
An operator who wants a header forwarded must say so; the default is that
nothing propagates.

Deleting the region also deleted the last variable-shaped part of the layout.
Word 2 and the descriptor pairs are gone, `TenantView::iter`, `find_by_name`,
`CarriedValue` and `RequestContextBuilder` went with them, and reading a value
is now a slot index. What survives of naming is `bag: true`, which encodes the
*registry* key name -- still a declared name, never a captured one.

Each component below needs one change to get there.

### `receiver:otlp`, `receiver:kafka`

Capture rules become extractors. `store_as` was already a rename from wire name
to logical name, which is exactly what `key` does:

```yaml
# Before
header_capture:
  headers:
    - match_names: ["x-tenant-id"]
      store_as: tenant_id
    - match_names: ["x-request-id"]

# After
tenant_tokens:
  request_identity:
    extractors:
      - key: tenant_id
        transport_header: x-tenant-id
        retain: true
      - key: request_id
        transport_header: x-request-id
        retain: true
```

The `defaults` limits (`max_entries`, `max_name_bytes`, `max_value_bytes`) have
no equivalent and need none: the declared extractor set is the bound. A token
cannot exceed `MAX_TOKEN_KEYS`, and no unnamed header is stored at any size.

The one real loss is **duplicate header names**. A key holds one value, while
capture kept every occurrence. Nothing in this design routes on a repeated
header, so the prototype takes last-write-wins; a component that genuinely
needs multiplicity should say so before the region is deleted.

### `exporter:otlp_grpc`, `exporter:otlp_http`

Propagation policy becomes `tenant_headers`. The `named` selector maps
directly; `all_captured` has no equivalent by design, since it is the rule that
lets an undeclared name escape:

```yaml
# Before
header_propagation:
  default:
    selector:
      type: named
      named: [tenant_id]
    name: preserve

# After
tenant_headers:
  - key: tenant_id
    header: x-tenant-id
```

`otlp_http` currently has no `tenant_headers` at all, so it needs the same
egress map the gRPC exporter already has.

### `processor:partition`

Not a rename, and the only case that needs new machinery. The partition
processor synthesizes a value and publishes it by pushing a synthetic transport
header onto the context. It is developed as its own example below, in
[Forking context on a data attribute](#example-forking-context-on-a-data-attribute).

### `exporter:kafka`

Kafka routes and partitions by header name today, in two independent places:

```yaml
# Before
traces:
  topic_from_transport_header: x-tenant-id
  partition_by_transport_headers: true

# After
traces:
  topic_from_key: tenant_id
  partition_by_keys: [tenant_id]
```

Reading by key removes the pre-normalization dance in `topic_router.rs`, where
the configured name is lowercased at build time to match how capture stored
logical names. With key ids there is no name to normalize.

The kafka *receiver* is the mirror image: it captures message headers on
ingress, so it takes the extractor treatment above. Worth noting that its
headers are Kafka's, not HTTP's -- which is what the baseline means by
transport headers being generic, with each receiver deriving them from its own
protocol.

### `receiver:traffic_generator` and `crates/validation`

Both exist to exercise the pipeline, and both currently assert by name. They
move to declaring tokens like any other receiver, and asserting on
`slot_value(registry.value_slot(key))` rather than `find_by_name`. This is the
largest mechanical diff and the least interesting one.

### What has to be decided first

- **Duplicate names.** Confirm nothing needs them, or add a multi-value key.
- **`sensitive: true`.** Capture has a flag for headers like `authorization`.
  Token keys have no such marker, and the natural answer is that a secret
  should never become a key at all -- it should reach an authorization
  extension and contribute a derived fact instead.
- **Skip statistics.** Capture reports headers dropped for exceeding limits.
  The token equivalent is an unresolved token, which is observable but means
  something different.

## Component integration patterns

Everything above this point is implemented. **Everything below is a design
sketch**: proposed integrations for components that do not yet consume tenant
tokens, recorded so the pattern can be reviewed before it is built.

The four components already covered illustrate four distinct ways a component
can consume a tenant token:

| Pattern | Component | What the token does |
| --- | --- | --- |
| **Create** | `receiver:otlp` | Declares extractors; mints identity |
| **Map** | `exporter:otlp_grpc` | Selects an outbound wire name |
| **Select** | `processor:batch` | Chooses a partition and its parameters |
| **Scope** | `exporter`/`receiver:topic` | Gates what crosses a boundary |

The remaining components are worth working through only where they introduce a
*new* pattern. A component that merely re-instantiates "create" or "select"
adds configuration surface but no design insight. The examples below are
therefore ordered by the pattern they introduce, and the list ends when new
components stop introducing new patterns.

### Example: authenticated identity as a token source

**The problem with the examples so far.** Every token above resolves
`customer_id` from `x-tenant-id`, a header the client chose. On a trusted
internal network that is fine. On an open ingest endpoint it is a tenant
impersonation vulnerability: any client can claim to be any tenant, and every
routing, partitioning and quota decision downstream inherits the lie. A header
is a *claim*; it is not evidence.

The engine already has the missing piece, and the baseline design already
names it: "Authorization extensions are the source of trusted material for
defining tenant tokens." The `bearer_token_authorizer` capability
(`crates/engine/src/capability/auth/`) authenticates an inbound token and emits
an `AuthorizedIdentity { subject, audience }`. Its own documentation draws
exactly the line this design needs:

> It admits on the token alone; it does not perform contextual, per-request
> authorization (route, tenant, signal, or action scoping), which needs request
> context it never receives and belongs downstream -- consuming the
> `AuthorizedIdentity` this capability emits.

Tenant tokens are that downstream consumer. The authorizer decides *whether the
caller is who they say they are*; tenant tokens decide *what that principal is
allowed to be called, and where their data goes*.

#### The extractor

The baseline names extractors after the source they read from, in pairs: an
extracting form and a matching form (`transport_header` /
`transport_header_match`, `resource_attribute` / `resource_attribute_match`).
Authorization material follows the same pattern:

- `authorized_attribute: <name>` copies a field asserted by the request's
  authorization extension into a token key.
- `authorized_attribute_match: <name>` with `value:` gates resolution on that
  field without contributing a key.

Mechanically this is the cheapest kind of extension. Like `remote_address` it
is a per-request value from a non-header source, so the build and probe phases
are untouched.

#### One token, two provenances

The useful shape is a single token whose keys come from different sources,
because that is what lets an untrusted claim be interpreted only in the context
of a trusted one:

```yaml
tenant_tokens:
  customer_project:
    extractors:
      # Evidence: asserted by the authorization extension.
      - key: customer_id
        authorized_attribute: enduser.id
        retain: true
      # Claim: chosen by the client.
      - key: project_id
        transport_header: x-project-id
        retain: true
```

`customer_id` is evidence and `project_id` is a claim, but they are keys of one
token, so every condition that tests `project_id` also has `customer_id`
available to test alongside it. A client can name its own projects; it cannot
name another customer's project, because the customer half of the pair is not
theirs to choose.

All-or-nothing resolution does the rest. A request with no authorization, or
with no `x-project-id`, resolves no token, so no downstream condition matches
it and it takes the default. Fail-closed is not new machinery here; it is the
existing token semantics applied to a token that happens to mix sources.

#### Enumerating the permitted pairs

A token resolves keys; it does not validate relationships between them. The
conditions do, because a condition over two keys is precisely a join on those
two keys, which is what the hash join already compiles:

```yaml
topic_router:
  type: exporter:topic
  config:
    routing:
      tenant_tokens: [customer_project]
      routes:
        # Acme's own projects.
        - entries:
            - { key: customer_id, value: acme }
            - { key: project_id, value: checkout }
          topic: acme_checkout
        - entries:
            - { key: customer_id, value: acme }
            - { key: project_id, value: search }
          topic: acme_search
        # Any other project acme names is still acme's.
        - entries:
            - { key: customer_id, value: acme }
            - { key: project_id }
          topic: acme_shared
      default_topic: unmatched
      export:
        allow_keys: [customer_id, project_id]
```

The third route is the wildcard form from the baseline: `project_id` with no
`value` requires the key to be present without constraining it. Its position
matters, since first-match-wins means it catches only what the two specific
routes did not.

Cost is unchanged by the extra key. Both keys belong to one signature, so the
whole check is still one symbol word at resolve time and one integer lookup per
probe. The authorization matrix is data in a hash table, not a chain of
comparisons, and it stays flat as customers and projects are added.

Two details are worth calling out in configuration review:

- `default_topic` is doing security work. Pointing it at a real backend
  instead of a quarantine sink silently converts a deny into an allow. This is
  the one place where first-match-wins with a fallback is dangerous rather
  than convenient.
- `export.allow_keys` lists both keys here, so both cross the boundary. Listing
  only `project_id` would let a tenant pipeline partition by project while
  keeping the customer identity from reaching a backend that has no business
  knowing it.

#### Binding the authorizer

The receiver needs no new configuration surface. It declares the authorization
extension and binds it through the existing `capabilities` block, exactly as
the Azure Monitor exporter binds `bearer_token_provider` today:

```yaml
extensions:
  cluster_auth:
    type: extension:oidc_authorizer
    config:
      issuer: https://kubernetes.default.svc
      audience: otel-collector

nodes:
  otlp_receiver:
    type: receiver:otlp
    capabilities:
      bearer_token_authorizer: cluster_auth
    config:
      protocols:
        grpc:
          listening_addr: 127.0.0.1:4317
```

Binding the capability is what fixes the ordering, and the ordering is the
security property: the authorizer runs first, a deny ends the request before
resolution runs at all, and only then does the engine resolve tokens over the
fields the identity asserted.

#### Hypothetical integration

The full path, from credential to outbound header name:

```yaml
tenant_tokens:
  # Ingress identity: one authenticated key, one claimed key.
  customer_project:
    extractors:
      - key: customer_id
        authorized_attribute: enduser.id
        retain: true
      - key: project_id
        transport_header: x-project-id
        retain: true

  # Downstream identity, resolved locally from what the boundary admitted.
  routed_tenant:
    extractors:
      - key: customer_id
        imported_key: customer_id
        retain: true
      - key: project_id
        imported_key: project_id
        retain: true

groups:
  secure_ingest:
    topics:
      acme:
        description: Authenticated traffic for the acme customer.
      unmatched:
        description: Quarantine for unresolved or unauthorized requests.

    pipelines:
      ingress:
        extensions:
          cluster_auth:
            type: extension:oidc_authorizer
            config:
              issuer: https://kubernetes.default.svc
              audience: otel-collector

        nodes:
          otlp_receiver:
            type: receiver:otlp
            capabilities:
              bearer_token_authorizer: cluster_auth
            config:
              protocols:
                grpc:
                  listening_addr: 127.0.0.1:4317

          topic_router:
            type: exporter:topic
            config:
              routing:
                tenant_tokens: [customer_project]
                routes:
                  - entries:
                      - { key: customer_id, value: acme }
                      - { key: project_id }
                    topic: acme
                default_topic: unmatched
                export:
                  allow_keys: [customer_id, project_id]

        connections:
          - from: otlp_receiver
            to: topic_router

      acme_egress:
        nodes:
          from_topic:
            type: receiver:topic
            config:
              topic: acme
              subscription:
                mode: broadcast
              tenant_context:
                import:
                  allow_keys: [customer_id, project_id]
                tokens: [routed_tenant]

          batcher:
            type: processor:batch
            config:
              max_batch_duration: 200ms
              # One partition per project, so a batch never mixes them and
              # the exporter always has a single tenant context to name.
              batch_by:
                tenant_tokens: [routed_tenant]
                partitions:
                  - entries:
                      - { key: customer_id, value: acme }
                      - { key: project_id, value: checkout }
                  - entries:
                      - { key: customer_id, value: acme }
                      - { key: project_id, value: search }

          otlp_exporter:
            type: exporter:otlp_grpc
            config:
              grpc_endpoint: http://backend-acme:4317
              tenant_headers:
                - key: customer_id
                  header: x-scope-orgid
                - key: project_id
                  header: x-acme-project

        connections:
          - from: from_topic
            to: batcher
          - from: batcher
            to: otlp_exporter
```

Note what is absent from every block above: the credential. `BearerToken` is a
secret-protecting wrapper and is never exposed to token resolution, so the
packed request context cannot carry a bearer token into a batch, across a
topic, or out to a backend. The identity derived from a secret propagates; the
secret does not.

The value reaching `x-scope-orgid` is now an authenticated fact, while the one
reaching `x-acme-project` is a client claim scoped by it. Nothing downstream of
the receiver changed to get that property, because downstream nodes name keys
rather than wire locations -- which is exactly why the same pipeline serves a
header-identified deployment and an authenticated one.

#### Implementation note

Authorization extensions reach token resolution through a sink, a narrow
interface to the compiled machine:

```rust
/// Authorization extensions assert semconv-named facts about the caller.
/// They never see the compiler, the scratch, or the packed layout.
pub trait IdentitySink {
    fn offer(&mut self, key: &str, value: &[u8]);
}
```

The extension contributes inputs; only the engine declares them complete, which
it must, since symbol words are valid only once every input is known. A
borrowed slice out of an already-parsed certificate or JWT goes straight into
the scratch arena, so the one-allocation-per-request property survives. Fields
no token declared an extractor for fail the key lookup and are dropped, so what
an authorizer can assert and what a pipeline's tokens want need no coordination
and can be configured by different teams. The sink can also be wrapped to
restrict an extension to a name prefix it is authorized to assert, so a
compromised mTLS extension cannot assert `enduser.id` and hijack routing.

The existing `bearer_token_authorizer` API does not change:
`AuthorizedIdentity { subject, audience }` is adapted into the sink by engine
glue, offering `subject` as `enduser.id`. Extensions wanting to assert richer
facts opt into a separate capability whose vocabulary is the sink itself.

### Example: forking context on a data attribute

`processor:partition` splits one batch into N by evaluating an OPL expression
over the payload, then publishes the partition value so downstream nodes can
act on it. Today it publishes by pushing a synthetic transport header, which
puts it squarely in the capture path.

It is the same shape as the topic exporter/receiver pair. Both are **context
forks**: a node consumes one context and constructs new ones, and both halves
of the question -- what survives, what is created -- have to be answered.

| | topic exporter/receiver | `processor:partition` |
| --- | --- | --- |
| fork driven by | condition over keys | expression over payload |
| fan-out | 1 of K fixed outputs | N, determined by data |
| crosses | a pipeline boundary | nothing |
| what survives | `export`/`import` lists | everything |
| what is created | the receiver's tokens | one key |

The difference in the last two rows is the whole point. The topic pair filters
because it crosses a scope: a pipeline-scoped token must not escape its
pipeline, so both ends carry an allowlist. The partition processor crosses no
scope, so nothing needs filtering and nothing needs declaring on that side.
**It is the topic pair minus the boundary.** The fix to the current code is
that simple: forward the inbound context to each output instead of building a
fresh one, then write the key.

#### Partitioning is what makes a data attribute liftable

The baseline restricts `resource_attribute` extractors to single-resource
batches, for a good reason: a key holds one value, and a batch generally has
many values for any given attribute. There is no correct answer to "what is the
customer id of this batch".

Partitioning is the operation that establishes the missing precondition. After
a split on `resource.attributes["customer.id"]`, each output batch has exactly
one value for it by construction. So this processor is not merely *allowed* to
lift a payload attribute into the context -- it is the node that makes doing so
well defined:

```yaml
tenant_tokens:
  partitioned_tenant:
    extractors:
      - key: edge_tenant
        transport_header: x-tenant-id
      - key: customer_id
        node_defined: splitter
        retain: true

nodes:
  splitter:
    type: processor:partition
    config:
      partition_by:
        opl_expression: resource.attributes["customer.id"]
      header_serialization_strategy: json
    out_ports:
      default: [sink]

  sink:
    type: exporter:otlp_grpc
    config:
      grpc_endpoint: http://collector:4317
      tenant_headers:
        - key: customer_id
          header: x-acme-customer
```

A node-defined key is a third writer, so it takes a binding of its own under
[one key, one value](#one-key-one-value): a key defined by a node cannot also
be extracted from a header, and two nodes cannot define the same key.

`node_defined: splitter` is the only declaration. The engine resolves it at
build time and hands the processor a slot index, so the processor writes a
slot, never a name -- the same compile-out that extractors get. The exporter
names it on egress, exactly as it would name a header-derived key. Nothing in
between knows the string `customer.id` or `x-acme-customer`.

`header_serialization_strategy` survives unchanged. The problem it solves --
an OPL expression yields ints, doubles, bools or null, and a slot holds bytes
plus a kind -- is identical whether the destination is a header or a value
slot.

#### Deferred resolution

A token containing a node-defined key cannot resolve at the receiver, because
one of its keys does not exist yet. This is the one piece of machinery the
example requires, and the layout already accommodates it: the resolved-token
bitmask simply keeps that token's bit clear until the defining node runs and
completes the resolve.

Two consequences follow.

A condition over `partitioned_tenant` is meaningless upstream of `splitter`,
and evaluating one there is a configuration error rather than a miss. This is
the same reachability analysis the baseline already requires for scoping pair
slots, applied to node order rather than graph connectivity.

And the fork itself must pay for the resolve: N outputs means N symbol words
per allocated pair, not one. That is the honest cost of a data-driven fork, and
it argues for keeping the number of tokens carrying node-defined keys small.

#### The cheap case

If `customer_id` is only used to choose a Kafka topic and never emitted as a
header, drop `retain: true`. It gets no value slot, and the fork writes a
symbol word per output and copies zero bytes. Partitioning on an attribute and
routing on it is then free of any per-value allocation -- which is the same
payoff match-only keys give everywhere else, arriving here without the design
having to do anything special.

#### The bug this replaces

Worth recording why the current code cannot be left alone. The single-partition
path reuses `inbound_context`, but the multi-partition path builds a
`Context::default()` and copies over only transport headers and peer address.
The tenant context is therefore preserved or destroyed depending on how the
data happened to partition. Making the processor a declared definition site
removes the fork in the code along with the fork in behaviour.

## Where the code lives

Paths below are relative to `crates/`.

- User-facing config types --
  `config/src/tenant.rs`
- Build, resolve, probe and pack --
  `config/src/tenant/compiled.rs`
- `Context.request`, accessors, merge helpers --
  `otap/src/pdata.rs`
- Registry compilation per group --
  `controller/src/lib.rs`
- Registry sharing --
  `engine/src/context.rs`
- Ingress resolution --
  `otap/src/otlp_http.rs`
- Routing and egress policy --
  `core-nodes/src/exporters/topic_exporter/mod.rs`
- Ingress policy and re-resolution --
  `core-nodes/src/receivers/topic_receiver/mod.rs`
- Partitioning and context merge --
  `core-nodes/src/processors/batch_processor/mod.rs`
- Outbound header naming --
  `core-nodes/src/exporters/otlp_grpc_exporter/mod.rs`

## Prototype limitations

- No tests were written for this prototype; validation was by full-suite
  regression plus live end-to-end runs.
- Token resolution is wired into the OTLP/HTTP receiver only. Other receivers
  still use the capture policy. `TokenInputs` is the extension point: new
  sources of tenant material are added there rather than at every receiver.
- Only `exporter:otlp_grpc` implements `tenant_headers`.
- The name-bearing capture region still exists in the packed layout, so a wire
  name can still reach an egress boundary without being declared. Closing that
  hole is the migration described in
  [Retiring the capture policy](#retiring-the-capture-policy).
- Conditions are compiled against the union of all conditions declared in a
  group; graph reachability is not considered, so an unreachable node still
  contributes pair slots. The baseline requires the reachable cross-product:
  "for every token and every reachable condition (the conditions of limiters
  reachable from the node)".
- Tokens are declared only at engine top level. The baseline also allows
  group-level and pipeline-level declarations, so that a token defined in a
  pipeline cannot escape it and the engine enforces that discipline
  automatically. In this prototype the only thing preventing escape is the
  pair of boundary allowlists, which is a policy the operator must write
  rather than a scope the engine derives.
- Retention is declared per extractor and applies group-wide. Liveness over the
  pipeline graph could derive it instead, since a key no reachable node reads
  need not be packed at all. Liveness may only shrink the propagated set, so it
  composes with the boundary allowlists without weakening them.
- Four extractor kinds exist: `transport_header`, `generic_key`,
  `remote_address` and `imported_key`. The baseline also lists `receiver_id`,
  `source_node_id`, `masked_remote_address`, `transport_header_match`,
  `resource_attribute` and `resource_attribute_match`. The `_match` forms
  matter beyond convenience: they gate token resolution without contributing a
  key, which is how the baseline expresses "this token applies only to legacy
  clients".
- The reserved token key `signal` is not implemented.
- The registry `epoch` is carried in every packed context so a stale context
  can be detected after a reconfigure, but nothing acts on it yet.
