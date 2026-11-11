---
Proposal Name: query-engine batch metadata manipulation
Start Date: 2026-11-03
RFC PR: open-telemetry/otel-arrow#4632
Tracking Issue: open-telemetry/otel-arrow#3310
---

# RFC 0010: OPL/OTAP Query Engine Batch transformation

## Summary

This RFC defines the capabilities for reading values from batch metadata in OPL
and the OTAP query engine.

A collection of functions will be added to inspect the contents of batch
metadata and retrieve values.

This RFC does _not_ define any capability for updating OTAP batch metadata
values. Such design work is left for some future RFC.

## Motivation

RFCs 0006 and 0004 define the logical structure and policies around batch
metadata propagation. After batch metadata has been populated, the next
question is "what can I do with this batch metadata?"

One reasonable answer may be that we wish to control the flow and/or content
of telemetry in response to certain metadata values.

For example, I may wish that some distinct category of telemetry
batches be routed to a specific destination. The category membership
predicate may be complex -- it be may expressible wholly or only partially by
inspection of the metadata, may also involve transformations of the metadata
values and may involve logical combination of predicates involving other
(non-metadata related) fields/expressions.

For example:

```js
signals |
if (
    has_transport_header("x-my-header")
    // predicate involves other non-metadata fields
    and resource.attributes["x"] == Y /* and|or|not ... */
) {
    route_to "destination_a"
}

if (
    // predicate involving complex logic applied to value
    coalesce(
        regexp_capture(
          authorized_identity_claim_value("sub"),
          "^sub:([0-9]+).*",
          1
        ),
        ""
    ) == "123456789"
) {
    route_to "route_for_sub_123456789"
}
```

We may wish to control the _content_ of telemetry as well, again using complex logic

```js
signals |
if (resource.attributes["network.peer.address"] == null and metadata_has_peer_addr()) {
    // ensure peer port always corresponds to address by setting both at same time
    set resource.attributes["network.peer.address"] = peer_addr_from_metadata() |
    // extract port
    set resource.attributes["network.peer.port"] = coalesce(regexp_capture(
        peer_addr_from_metadata,
        ".*:([0-9]+).*",
        1
    ), "0") as Integer
}
```

Trying to express such operations through YAML pipeline configuration may
become untenable. This complex behavioural logical is clearly best expressed
programmatically.

## Guide-level explanation

### Background: Context (Logical) Data Model

This section describes the context's "logical" data model. The context
metadata may be physically organized in a very different manner than
what is described below.

An OTAP batch's metadata is composed of named fields which are grouped into
different "_domains_", which can be understood as different "types" of
metadata. There are currently five such domains:

- Authorized Identity - contains claims resolved from an authorization method
  (e.g. claims from a JWT Token). Each claim contains a name (of type `String`)
  and zero, one or multiple values which are also of type `String`.
- Transport Headers - contains headers resolved from a received batch. These
  contain both a "name" and optionally an "original name" (both of type
  `String`) and a single value which may be of type `String` or `Binary`.
  The header "name" is the identifier we've chosen to use to store the header
  in the batch metadata whereas the "original name" is the name of the actual
  header when the data was received by the receiver.
  Note that pdata metadata preserves duplicate headers, so logically a header
  with a given name can have more than one value.
- Network Info Peer address - contains the IPV4/V6 socket address from which
  the batch was received by a network facing receiver.
- Constant/Random - static or computed values, populated by the engine based
  on context entry policy configuration. These have a name and a `String`
  value.

The various fields belonging these "domains" can be grouped into "_context
entries_" via policies. Context entries are named, and composed of metadata
values from the various domains called "_members_":

```yaml
policies:
  context:
    entries:
      # a context entry named "example_one"...
      #
      # The "entry" contains a list of members. Each member has a type
      # which identifies the "domain", a "name" which represents both
      # the member's name within this context entry _and_ the metadata
      # field within its domain containing the value.
      #
      # Some members also have a "value" field which is interpreted
      # in a context dependant way depending on the "type".

      example_one:
      - # this context entry member references a transport header whose
        # name is "albert-header". Note, this is the header "name" which
        # is different than the "original name"
        type: transport_header
        name: "albert-header"

      - # entry referencing an authorized identity claim "sub"
        type: authorized_identity
        name: "sub"

      - # constant whose name is "my-const", which has a value of "foo"
        type: constant
        name: "my-const"
        value: "foo"

      - # a "random" context entry, whose value is a generated uuidv7
        type: randomness
        name: "my-random"
        value: uuidv7

      # this context entry includes the socket address
      - type: network_info
        name: peer_socket_addr
```

The member list within a context entry also defines whether a context entry
is present at all. A context entry is only present in the batch if all its
members are present.

There is also a special meta context entry member that
controls whether a context entry should be present in a given batch called
`transport_header_match`:

```yaml
policies:
  context:
    entries:
      # This batch will only have a context entry called "example_two" if
      # it contained a transport header named "my-header-2" and a value of
      # "some-val"
      example_two:
      - type: transport_header_match
        name: my-header-2
        value: some-val
```

### Challenges

There are challenges of working with this such a model. There are a variety of
disparate types which any particular metadata value may take on. It may be a
`String`, or `Binary` or a list of `String`s (this ignores peer address, which
internally is a `SocketAddr`, which we need to decide how to handle).

What's more, the logic for determining _which_ type any given field _could_ be
depends on its domain. This is challenging because the domain model may evolve
over time. For example, imagine some code written as:

(This is pseudocode - not the actual proposed API)

```js
if (context["field"].domain == "authorized_identity") {
    // handle String and []String
} else if (context["field"].domain == "transport_header") {
    // handle String and Binary
} else {
    // Now we _know_ it's a String
}
```

This code could break if we adapt the data model of context to add new types
and domains.

To help maintain greater program compatibility as the context data model
evolves, and to reduce program complexity, we should _avoid forcing_ the
users to deal with complexity and attempt to encapsulate the data model
behind a simpler facade.

## Reference-level explanation

### Query-Language API

The utilities for reading context values will be available as a collection of
functions whose signatures will be presented in the following section.

Here are some examples of using some of these functions:

```js
// filter out all logs that don't have some context entry
logs | where has_context_entry("my_context_entry")

// set the value of a context entry member as a resource attribute
// (but only if it is a string type)
signals |
    if is_context_entry_member_string("my_context_entry", "some_member") {
        set resource.attributes["my.attr"] = context_entry_value(
          "my_context_entry",
          "some_member"
        )
    }


// route telemetry received from some specific identity to a given destination
signals |
    if authorized_identity_claim_has_value("sub", "sub:123456789") {
        route_to "special_route"
    }

// filter out non-sampled spans
signals |
    if (is Span) {
        where (
          has_transport_header("b3") and regex_capture(
              transport_header_value("b3"),
              "[a-z0-9]+-[a-z0-9]+-(.)-.*",
              1
          ) == "1"
        ) or transport_header_value("X-B3-Sampled") == "1"
    }
```

Note that the convention I'm using here is that:

- `?` in parameters means the argument is optional (the function implementation
  will assume a default value for these parameters)
- `?` on return types means the type may be null
- `(T1 | T2 | T3 | ... TN)` types means the type may be either `T1` or
  `T2` or `T3` ... or `TN` (i.e. the parentheses do not indicate a tuple).
- `[]T` means the type is an array of `T`

<!--
I've also used "rs" as the language for all the function signatures below
simply because I wanted the syntax highlighting. These are not necessarily Rust
functions we'll actually write - they're signatures of functions that will be
available in KQL/OPL/OTTL functions executed by the OTAP query engine.
-->

#### Context Functions

```rs
has_context_entry(entry_name: String) -> Boolean
```

Returns `True` if the OTAP batch contains a context entry of the given name,
otherwise returns `False`.

---

```rs
has_context_entry_member(entry_name: String, member_name: String) -> Boolean
```

Returns `True` if the OTAP batch contains a context entry with the given name
and if this contains a member with the given name, otherwise returns `False`.

---

```rs
has_context_entry_member_value(
   entry_name: String,
   member_name: String,
   value: String | Binary
) -> Boolean
```

Returns `True` if the OTAP batch contains a context entry with the given name
which contains a member of the given name, and the member's value either equals
or contains the passed value. Returns `False` otherwise.

If the passed `value` is a `Binary`, it can only be matched for equality
against the value any Transport Headers having the same name as the passed
`member_name` parameter.

If the passed `value` parameter is a `String`:

- If the member belongs to the an Authorized Identity domain and...
  - If the claim value contains multiple values, this function will
    return `True` if any claim value is equal to the string, otherwise it
    returns `False`.
  - Else if the claim contains zero values, this function returns `False`
- If the member belongs to the Transport Header domain and ...
  - If the value of the Transport Header is `Binary`, this returns `False`
- Else the claim contains a single `String` value and so it will be matched
  for equality against the passed `value` parameter.

Note that if the member is network peer address, the peer address will be
converted to a string by calling
[`SocketAddr::to_string`](https://docs.rs/lib/latest/lib/net/enum.SocketAddr.html#impl-ToString-for-T)
before comparing the string value.

---

```rs
is_context_entry_member_string(entry_name: String, member_name: String) -> Boolean
```

Returns `True` if the OTAP batch contains a context entry with the given name
which contains a member with the given name and the value of this member is a
scalar `String`, returns `False` otherwise.

If the member is of the Authorized Identity, this returns `False` if there are
zero claim values, or if there are multiple claim values. This is because this
function is used to determine if the value is logically a scalar string.

If the member is of the Transport Header domain, this returns `False` if the
value is `Binary`.

This will return `True` if the member is the network peer address.

---

```rs
is_context_entry_member_binary(
    entry_name: String,
    member_name: String
) -> Boolean
```

Returns `True` if the OTAP batch contains a context entry with the given name
which contains a member with the given name and the value of this member is a
scalar `Binary`, returns `False` otherwise.

Since only the Transport Header domain can contain binary values, this will
return `False` if the context entry member belongs to any other domain.

If there are multiple transport headers containing a given header, this returns
`False`. This is because this function is used to determine if the value is
logically a scalar `Binary`.

---

```rs
is_context_entry_member_scalar(
    entry_name: String,
    member_name: String
) -> Boolean
```

Returns `True` if the OTAP batch contains a context entry with the given name
which contains a member with the given name and the value of this member is a
scalar `String` or `Binary`, returns `False` otherwise.

If the member is of the Authorized Identity domain, this returns `False` if
there are zero claim values, or if there are multiple claim values.

For all other domains, this will return `True` since no other domains' members
can contain zero/multiple values.

---

```rs
context_entry_value(
    entry_name: String,
    member_name: String
) -> (String | Binary | []String | []Binary)?
```

Returns the value of the context entry member.

If the OTAP batch does not have a context entry with this name, or the context
entry does not have such a member, this will return `Null`.

If the member is of the Authorized Identity domain:

- If there are zero claim values this will return `Null`
- If there are multiple claim values, it will return type `[]String`
- Otherwise (there is a single claim value) it will return type `String`

For the Transport Header domain, this will return either `String` or `Binary`
type depending on the value of the header if there is only a single header.
If there are multiple headers with the same name, and all of them are `String`
then this will return type `[]String`, else  this will return type `[]Binary`.

For all other domains, this returns type `String`.

For the network peer address domain, this will return the peer address converted
to a string by calling
[`SocketAddr::to_string`](https://docs.rs/lib/latest/lib/net/enum.SocketAddr.html#impl-ToString-for-T)

---

```rs
context_string_value(entry_name: String, member_name: String) -> String?
```

Returns the same values as `context_entry_value` when this function would
return a `String`, and `Null` otherwise.

This means that for Authorized Identity members with zero or multiple values,
as well as for Transport Header members with binary values or if there are
multiple headers with the same name this returns `Null`.

```rs
context_binary_value(entry_name: String, member_name: String) -> Binary?
```

Returns the same values as `context_entry_value` when this function would
return a `Binary`, and `Null` otherwise.

Effectively, this means this will _only_ return a non-`Null` value for members
of the Transport Header domain that contain a `Binary` value. However if there
are multiple headers with the same name, then this will return `Null`.

#### Transport Header Functions

_Note on the parameters_:

All of the functions below take the `header_name` parameter which is used to
identify the header that is the target of the function. This will be matched
against the header's "name" (e.g. what the header is "stored as" in the context
metadata), which may be different than the header's "original name" (e.g. the
actual header name received alongside the OTAP batch).

To target the Transport Header by its "original name" instead of it's "stored
as" name pass the value `True` to the optional `original_name` parameter. This
parameter may be omitted, and when it is omitted it will default to `False`.

---

```rs
has_transport_header(
    header_name: String,
    original_name: Boolean?
) -> Boolean
```

Returns `True` if the OTAP batch contains the Transport Header identified by
the `header_name` parameter, otherwise returns `False`.

---

```rs
is_transport_header_string(
    header_name: String,
    original_name: Boolean?
) -> Boolean
```

Returns `True` if the value of the header identified by the `header_name`
parameter is a `String` type.

This returns `False` if the header is not a string type (e.g. it has a `Binary`
value), or if the header is not present in the OTAP Batch's context.

If there are multiple transport header with the same name, this will return
`False` because logically the value is not a scalar `String`.

---

```rs
is_transport_header_binary(
    header_name: String,
    original_name: Boolean?
) -> Boolean
```

Returns `True` if the value of the header identified by the `header_name`
parameter is a `Binary` type.

This returns `False` if the header is not a binary type (e.g. it has a `String`
value), or if the header is not present in the OTAP Batch's context.

If there are multiple transport header with the same name, this will return
`False` because logically the value is not a scalar `Binary`.

---

```rs
transport_header_value(
    header_name: String,
    original_name: Boolean?
) -> (String | Binary | []String | []Binary)?
```

Returns the value of the transport header. This may return either `String` or
`Binary` type depending on the type of the value of the header.

If there are multiple headers of the same matched name, if all the values are
of type `String` then this will return type `[]String` otherwise it will return
type of `[]Binary`.

If the header is not present, this returns `Null`.

---

```rs
transport_header_string_value(
    header_name: String,
    original_name: Boolean?
) -> String?
```

Returns the value of the transport header.

This returns `Null` if the transport header is not present, or if it is present
but value type is not `String` (i.e. it a `Binary` value).

This will also return `Null` if there are multiple duplicate headers with the
passed header name.

---

```rs
transport_header_binary_value(
    header_name: String,
    original_name: Boolean?
) -> Binary?
```

Returns the value of the transport header.

This returns `Null` if the transport header is not present, or if it is present
but value type is not `Binary` (i.e. it a `String` value).

This will also return `Null` if there are multiple duplicate headers with the
passed header name.

---

```rs
is_header_value_scalar(header_name: String, original_name: Boolean?) -> Boolean
```

This will return `True` if the header with the given name is present and if
there are no duplicate headers with this name.

#### Authorized Identity Functions

```rs
has_authorized_identity_claim(claim_name: String) -> Boolean
```

Returns `True` if the OTAP Batch's context contains an authorized identity
claim with the given name, and returns `False` otherwise.

Note - this returns `True` any time the claim is present, even when it has
zero values.

---

```rs
is_authorized_identity_claim_scalar(claim_name: String) -> Boolean
```

Returns `True` if the claim is present and contains a single value.

Returns `False` if the claim is not present, or if it is present but contains
either zero or multiple values.

---

```rs
authorized_identity_claim_has_value(
    claim_name: String,
    value: String
) -> Boolean
```

Returns `True` if the claim is present and is either equal to/contains the
passed `value`.

If the claim contains multiple values, this returns `True` if any of the values
is equal to the passed `value` parameter, and `False` otherwise.

If the claim contains only a single value then this returns `True` if the
passed `value` is equal to the single claim value.

 If the claim is absent or contains zero values, this returns `False`.

---

```rs
authorized_identity_claim_value(claim_name: String) -> (String | []String)?
```

Returns the value of the authorized identity claim if present.

If the claim is not present this returns `Null`.

If the claim contains zero values, this returns `Null`

If the claim contains a single value, this returns a type `String`

Otherwise the claim contains multiple values and this returns type `[]String`.

---

```rs
authorized_identity_claim_string_value(claim_name: String) -> String?
```

Returns the value of the authorized identity claim if present.

If the claim is not present this returns `Null`.

If the claim contains zero values or multiple values, this returns `Null`.

If the claim contains a single value, this returns a type `String`

#### Network Info Functions

```rs
has_network_info_peer_address() -> Boolean
```

Returns `True` if the network info peer address is available the OTAP
batch context, and `False` otherwise.

---

```rs
network_info_peer_address_string_value() -> String?
```

Returns the value of the network info peer address as a `String` if this value
is present in the request context. The value will be converted into a string by
calling
[`SocketAddr::to_string`](https://docs.rs/lib/latest/lib/net/enum.SocketAddr.html#impl-ToString-for-T).

If this peer address value is not present, this returns `Null`.

### Call Site Availability

These functions will be available in OPL, KQL and OTTL.

In OPL, these will be available on pipelines of any data type, including
nested pipelines of attributes and metric data points. E.g., the following
OPL program would be legal:

```rs
signals | apply attributes {
    if (key == "some_key" and is_context_entry_string("my_entry", "foo")) {
        // append the context value to the attribute
        set value = join(".", value, context_entry_string_value("my_entry", "foo"))
    }
}
```

## Rationale and alternatives

### Alternative 1: Do it with Processors

We could imagine bespoke processors, or configuration of existing processors,
that perform the same behaviour we're able to implement with transform programs
involving metadata. For example, the attribute processor could set attributes
from metadata values, the content router processor could route using metadata
values, and so on - and this behaviour could be configured through bespoke
yaml configuration of the processor component.

Even in the case where the routing predicate or assignment value expression is
very complex, it could either still be replicated via ever more intricate
configuration or alternatively, the transform processor could be chained with
the other processors to produce, in toto, the desired behaviour.

However, I feel that this alternative may not be sensible. I fear that with
this approach the bespoke configuration may become unwieldy and the combination
of processors needed for complex behaviour adds both overhead and pipeline
complexity.

### Alternative 2: A `context` field

Rather than having a collection of functions for accessing context, we could
have one or more fields available on all types in the query engine for
accessing context.

For example, perhaps we could have a "`context`" field that is a map, which
could be indexed by special keys containing both a domain and a name.

```rs
context["context_entry:entry_name"]

context["transport_headers:header_name"]

// and so on
```

This has some drawbacks --

First, there's ambiguity in the case that someone configured a context policy
whose name is literally one of the special identifiers we use for the domain.

```yaml
policies:
  context:
    entries:
      transport_headers: # uh oh!
      # ...
```

Of course we could have business logic to prevent this in our policy definition,
but it's also the case that we may in the future add new metadata domains
so we can't predict necessarily all the reserved context entry names.

The second drawback is that if OTel/OTAP ever did need to add a field called
`context`, we'd have a name conflict that would be difficult to sort out.

The third drawback is that there's less opportunity to hide the underlying
complexity of the context data model. For instance, this effectively forces
the type `context` type to be a `Map<String, (String | Binary | []String)?>`
whereas our function definitions can have more specific return types.

The fourth drawback is that, as we invent new ways to access context we may
need to invent new intricate and complex special keys. For example, consider
how we might access a transport header by it's "original name"? Something like
`context["transport_header:original_name:foo"]`. This is a messy problem that
can be completely side-stepped by the function call approach.

The only positive to this approach would be that we could use the _same_
identifiers in a hypothetical operator call that assigns context:

```kql
set context["some:key"] = "foo"
```

However at this stage, I'm not actually sure that we want to allow for the
flexibility in assigning context that such a syntax might enable, so I don't
think it's worth planning for. Also, nothing in the current design precludes
adding such a syntax in the future.

## Prior art

### OTel Collector's Attribute Processor Data Model

OTel Collector's Attribute Processor uses the fields `metadata` and `auth` to
reference batch metadata.

```yaml
  # Key specifies the attribute to act upon.
- key: <key>
  action: {insert, update, upsert}
  # FromContext specifies the context value to use to populate the attribute value.
  # If the key is prefixed with `metadata.`, the values are searched
  # in the receiver's transport protocol additional information like gRPC Metadata or HTTP Headers
  # (be sure to set `include_metadata: true` on the receiver).
  # If the key is prefixed with `auth.`, the values are searched
  # in the authentication information set by the server authenticator.
  # Refer to the server authenticator's documentation part of your pipeline for more information about which attributes are available.
  # If the key is `client.address`, the value will be set to the client address.
  # If the key doesn't exist, no action is performed.
  # If the key has multiple values the values will be joined with `;` separator.
  from_context: <other key>
```

Perhaps it begs the question why we don't do the same thing -- e.g. use
`metadata.<header_name>` as an expression to access transport header and
similarly `auth` to access authorized identity metadata.

This runs into the same problem discussed in the previous section with
respect to field name conflicts in the OTel Data model, although in this case
it's not simply hypothetical - `metadata` is the name of a field on metrics,
which in OTAP we call `attributes`, but this could still incite confusion.

## Unresolved questions

- What parts of the design do you expect to resolve through the RFC process
  before merge?
- What related issues are out of scope for this RFC and could be addressed
  independently later?

### Static resolution of context entries & program validity

Since context entries, transport header capture, etc. are part of the pipeline
configuration, presumably we may have a way to "know" statically the return
value of many of the functions.

For example, if we did not have a context policy that defines a context entry
called `"foo"` containing a member called `"bar"`, we know that calls such as
`has_context_entry("foo")` and `has_context_entry_member("foo", "bar")` will
always return `False` and similarly that calls such as
`context_entry_value("foo", "bar")` will always return `Null`.

I think for now it's fine if we let these resolve at runtime, but a performance
optimization may be to try to resolve them statically.

However, we can consider alternatives -- perhaps we want to say that such a
program would actually be _invalid_ in a context where no such policies are
defined. I can't think of a reason _why_ we might want to do this, but maybe
it's worth considering.

### Implicit header capture

We should know at configuration time whether a header capture policy has been
defined, which headers it preserves and what are the header names.

However, if no such policy has been configured, similarly to the case above
we may know statically that calls such as `has_transport_header("foo")` will
return `False` and that `transport_header_value("foo")` will return `Null`.

However, we may want to think more critically about the user's intention -
clearly they thought there was a possibility this header "foo" would be present
when they wrote the program, so it might be good to help them along either a)
warn them that they're accessing headers not defined in the capture policy or
b) automatically create a header capture policy based on the program they've
written.

## Future possibilities

_What natural extensions or follow-ups does this enable? This is a place to
record ideas that are out of scope now but worth remembering._

In the future we'll surely want the capability to set/update batch metadata
as opposed to what is defined in this PR which is simply reading it.
