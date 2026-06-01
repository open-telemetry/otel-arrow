# Attribute Operators

These operators work directly on attributes -- renaming keys, removing
attributes, or treating the attribute set as a stream for bulk filtering and
transformation. They work on log, span, and metric attributes as well as
resource and instrumentation scope attributes.

## Rename Attributes (`rename`)

The `rename` operator changes the key of an attribute. The syntax follows the
assignment pattern `new_key = old_key`: the destination key is on the left and
the source key is on the right.

```
// rename "http.method" to the newer semantic convention "http.request.method"
logs | rename attributes["http.request.method"] = attributes["http.method"]
```

Only records that have the source key are affected. If a record does not
contain the source attribute, it is left unchanged.

### Renaming resource and scope attributes

Resource and instrumentation scope attributes can be renamed using their
qualified paths:

```
logs | rename resource.attributes["service.namespace"] = resource.attributes["svc.namespace"]
```

```
logs |
rename
    instrumentation_scope.attributes["otel.scope.name"] =
        instrumentation_scope.attributes["scope.name"]
```

### Multiple renames

Multiple renames can be performed in a single operator invocation by separating
each assignment with a comma. Renames can span different attribute scopes:

```
logs |
rename
    attributes["http.request.method"] = attributes["http.method"],
    attributes["url.path"] = attributes["http.target"],
    resource.attributes["service.namespace"] = resource.attributes["svc.namespace"],
    instrumentation_scope.attributes["otel.scope.name"] =
        instrumentation_scope.attributes["scope.name"]
```

### Alias

`project-rename` is an alias for `rename`. The following is equivalent to the
first example above:

```
logs | project-rename attributes["http.request.method"] = attributes["http.method"]
```

### Errors

The following rename patterns are not allowed and will produce a
**"Duplicate key in rename target"** error:

Renaming an attribute to its own key (a no-op rename):

```
// error: source and destination keys are the same
logs | rename attributes["http.method"] = attributes["http.method"]
```

Multiple renames targeting the same destination key:

```
// error: two different sources both rename to the same destination
logs |
rename
    attributes["http.request.method"] = attributes["http.method"],
    attributes["http.request.method"] = attributes["method"]
```

## Remove Attributes (`exclude`)

The `exclude` operator removes attributes by key:

```
// remove a deprecated attribute
logs | exclude attributes["http.method"]
```

Only records that have the specified key are affected. Records without the
key are left unchanged.

### Removing resource and scope attributes

Like `rename`, `exclude` works on resource and instrumentation scope attributes:

```
logs | exclude resource.attributes["internal.tag"]
```

```
logs | exclude instrumentation_scope.attributes["debug.flag"]
```

### Removing multiple attributes

Multiple keys can be removed in a single invocation by separating them with
commas. Removals can span different attribute scopes:

```
logs |
exclude
    attributes["http.method"],
    resource.attributes["internal.tag"],
    instrumentation_scope.attributes["debug.flag"]
```

### Removing all attributes

If every attribute key is removed from a record, the attribute set for that
record is removed entirely:

```
// if a log only has "http.method" and "http.target", both are removed
// and the log will have no attributes at all
logs | exclude attributes["http.method"], attributes["http.target"]
```

### Alias

`project-away` is an alias for `exclude`. The following is equivalent to the
first example above:

```
logs | project-away attributes["http.method"]
```

## Apply to Attributes (`apply`)

The `apply` operator opens up an attribute set as a stream of individual
`key`/`value` pairs and runs a nested pipeline over them. This is useful when
you need to filter, transform, or conditionally process attributes based on
their keys or values -- rather than targeting a single attribute by name.

### Filtering attributes by value

Use `where` inside `apply` to keep or remove attributes based on their values:

```
// remove any attribute whose value matches a sensitive pattern
logs | apply attributes {
    where not(matches(value, ".*password.*"))
}
```

### Filtering attributes by key

Attributes can also be filtered by key. This is an alternative to `exclude`
when you need pattern-based removal rather than exact key matching:

```
// remove all attributes with keys starting with "internal."
logs | apply attributes {
    where not(starts_with(key, "internal."))
}
```

Multiple filters can be chained with `|` inside the `apply` block:

```
logs | apply attributes {
    where key != "http.method" |
    where not(matches(key, "debug\\..*"))
}
```

### Combining key and value filters

Key and value conditions can be combined with `and` and `or`:

```
// keep only attributes with specific keys that have non-empty values
logs | apply attributes {
    where (key == "http.request.method" or key == "url.path") and value != ""
}
```

### Modifying attribute values

Use `set value = ...` to transform attribute values in bulk:

```
// hash all attribute values
logs | apply attributes {
    set value = encode(sha256(value), "hex")
}
```

Arithmetic on values:

```
// increment all integer attribute values by 1
logs | apply attributes {
    set value = value + 1
}
```

Set all values to a static literal:

```
logs | apply attributes {
    set value = "redacted"
}
```

### Conditional processing

`if` blocks work inside `apply`, enabling per-attribute conditional logic
based on `key` or `value`:

```
// hash only sensitive attributes, leave others unchanged
logs | apply attributes {
    if (key == "user.email" or key == "user.ip") {
        set value = encode(sha256(value), "hex")
    }
}
```

```
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

```
logs | apply resource.attributes {
    where not(starts_with(key, "internal."))
}
```

```
logs | apply instrumentation_scope.attributes {
    where key != "debug.flag"
}
```

### Supported operators

The operators `where`, `set`, and `if` are supported inside `apply` blocks.
Operators like `rename` and `exclude` are not supported inside `apply` -- they
operate on the outer pipeline level by targeting specific attribute keys.
Using an unsupported operator inside `apply` will produce a planning error.

### Constraints

When using `set value = <expr>` where the expression references `value` (e.g.,
`set value = value + 1`), all attributes in the batch must have the same value
type. Mixing types (for example, some integer and some float attributes) will
produce an error. If the expression does not reference `value` (e.g.,
`set value = "redacted"`), this restriction does not apply.

If all attributes are filtered out by a `where` inside `apply`, the attribute
set is removed from the record entirely -- the same behaviour as `exclude` when
removing all keys.
