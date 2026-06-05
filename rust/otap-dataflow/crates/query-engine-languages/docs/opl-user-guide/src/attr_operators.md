# Attribute Operators

These special operators are available for processing attributes -- renaming
keys, removing attributes, or treating the attribute set as a stream for
filtering and transformation.

Currently support exists for log, span, and metric attributes as well as resource
and instrumentation scope attributes.

## Rename Attributes (`rename`)

The `rename` operator changes the key of an attribute. The syntax follows the
assignment pattern `new_key = old_key`: the destination key is on the left and
the source key is on the right.

```text
// rename "http.method" to the newer semantic convention "http.request.method"
logs | rename attributes "http.method" as "http.request.method"
```

Only records that have the source key are affected. If a record does not
contain the source attribute, it is left unchanged. If the destination attribute
already exists, its value will be replaced with the source attribute's value
(e.g. duplicate attributes will not be created by rename operations).

### Renaming resource and scope attributes

Resource and instrumentation scope attributes can be renamed using their
qualified paths:

```text
logs | rename resource.attributes "svc.namespace" as "service.namespace"
```

```text
logs | rename instrumentation_scope.attributes "scope.name" as "otel.scope.name"
```

### Multiple renames

Multiple renames can be performed in a single operator invocation by separating
each assignment with a comma:

```text
logs |
rename attributes "http.method" as "http.request.method", "http.target" as "url.path"
```

### Errors

The following rename patterns are not allowed and will produce a
**"Duplicate key in rename target"** error:

Renaming an attribute to its own key (a no-op rename):

```text
// error: source and destination keys are the same
logs | rename "http.method" as "http.method"
```

Multiple renames targeting the same destination key:

```text
// error: two different sources both rename to the same destination
logs |
rename attributes 
    "http.method" as "http.request.method",
    "method" as "http.request.method"
```

## Remove Attributes (`remove`)

The `remove` operator removes attributes by key:

```text
// remove a deprecated attribute
logs | remove attributes["http.method"]
```

Only records that have the specified key are affected. Records without the
key are left unchanged.

### Removing resource and scope attributes

Like `rename`, `remove` works on resource and instrumentation scope attributes:

```text
logs | remove resource.attributes["internal.tag"]
```

```text
logs | remove instrumentation_scope.attributes["debug.flag"]
```

### Removing multiple attributes

Multiple keys can be removed in a single invocation by separating them with
commas. Removals can span different attribute scopes:

```text
logs |
remove
    attributes["http.method"],
    resource.attributes["internal.tag"],
    instrumentation_scope.attributes["debug.flag"]
```

## Apply to Attributes (`apply`)

The `apply` operator opens up an attribute set as a stream of individual
`key`/`value` pairs and runs a nested pipeline over them. This is useful when
you need to filter, transform, or conditionally process attributes based on
their keys or values -- rather than targeting a single attribute by name.

### Filtering attributes by value

Use `where` inside `apply` to keep or remove attributes based on their values:

```text
// remove any attribute whose value matches a sensitive pattern
logs | apply attributes {
    where not(matches(value, r".*password.*"))
}
```

### Filtering attributes by key

Attributes can also be filtered by key. This is an alternative to `remove`
when you need pattern-based removal rather than exact key matching:

```text
// remove all attributes with keys starting with "internal."
logs | apply attributes {
    where not(starts_with(key, "internal."))
}
```

Multiple filters can be chained with `|` inside the `apply` block:

```text
logs | apply attributes {
    where key != "http.method" |
    where not(matches(key, r"debug\\..*"))
}
```

### Combining key and value filters

Key and value conditions can be combined with `and` and `or`:

```text
// keep only attributes with specific keys that have non-empty values
logs | apply attributes {
    where (key == "http.request.method" or key == "url.path") and value != ""
}
```

### Modifying attribute values

Use `set value = ...` to transform attribute values in bulk:

```text
// hash all attribute values
logs | apply attributes {
    set value = encode(sha256(value), "hex")
}
```

Arithmetic on values:

```text
// increment all integer attribute values by 1
logs | apply attributes {
    set value = value + 1
}
```

Set all values to a static literal:

```text
logs | apply attributes {
    set value = "redacted"
}
```

Note that the `key` of an attribute cannot be the target of the assignment.

### Conditional processing

`if` blocks work inside `apply`, enabling per-attribute conditional logic
based on `key` or `value`:

```text
// hash only sensitive attributes, leave others unchanged
logs | apply attributes {
    if (key == "user.email" or key == "user.ip") {
        set value = encode(sha256(value), "hex")
    }
}
```

```text
// set different values depending on the attribute key
logs | apply attributes {
    if (key == "log.level") {
        set value = "info"
    } else if (key == "log.source") {
        set value = "otel"
    }
}
```

### Resource and scope attributes

`apply` works on resource and instrumentation scope attributes by specifying
the qualified path:

```text
logs | apply resource.attributes {
    where not(starts_with(key, "internal."))
}
```

```text
logs | apply instrumentation_scope.attributes {
    where key != "debug.flag"
}
```

### Supported operators

The operators `where`, `set`, and `if` are supported inside `apply` blocks.
Operators like `rename` and `remove` are not supported inside `apply` -- they
operate on the outer pipeline level by targeting specific attribute keys.
Using an unsupported operator inside `apply` will produce an error.

### Constraints

When using `set value = <expr>` where the expression references `value` (e.g.,
`set value = value + 1`), all attributes in the batch must have the same value
type. Mixing types (for example, some integer and some float attributes) will
produce an error. If the expression does not reference `value` (e.g.,
`set value = "redacted"`), this restriction does not apply.

<!-- TODO Once supported, add note about using "if (value is <Type>){..." -->
