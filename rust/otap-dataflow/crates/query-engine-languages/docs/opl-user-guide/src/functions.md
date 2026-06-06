# Function Reference

OPL provides built-in functions for string manipulation, hashing, encoding,
math, date/time formatting, UUID generation, and null handling. Functions are
called with standard syntax: `function_name(arg1, arg2, ...)`.

Functions can be used anywhere an expression is accepted -- in `where`
conditions, `set` assignments, `apply` blocks, and `if` conditions.

## String Functions

### `contains(haystack, needle)`

Returns `true` if the string `haystack` contains the substring `needle`:

```text
logs | where contains(body, "error")
```

### `matches(haystack, pattern)`

Returns `true` if the string `haystack` matches the regular expression
`pattern`. The pattern uses standard regex syntax prefixed with `r`:

```text
logs | where matches(resource.attributes["k8s.pod.name"], r".*testing.*")
```

### `starts_with(string, prefix)`

Returns `true` if `string` starts with `prefix`:

```text
logs | where starts_with(attributes["url.path"], "/api/")
```

### `ends_with(string, suffix)`

Returns `true` if `string` ends with `suffix`:

```text
logs | where ends_with(attributes["url.path"], "/health")
```

### `lower_case(string)`

Converts a string to lowercase:

```text
logs | set attributes["http.request.method"] = lower_case(attributes["http.request.method"])
```

Note: `lower_case` only accepts string values. When operating on attributes
that may hold non-string types, guard with a type check:

```text
logs | where attributes["http.target"] is String and
    contains(lower_case(attributes["http.target"]), "/api/")
```

### `upper_case(string)`

Converts a string to uppercase:

```text
logs | set event_name = upper_case(event_name)
```

### `concat(a, b, ...)`

Concatenates two or more values into a single string:

```text
logs | set body = concat("[", severity_text, "]: ", event_name)
```

### `concat_ws(separator, a, b, ...)`

Concatenates values with a separator between each. `join` is an alias for
`concat_ws`:

```text
logs | set body = concat_ws(" - ", severity_text, event_name)

// equivalent using the alias
logs | set body = join(" - ", severity_text, event_name)
```

### `substring(string, start [, length])`

Extracts a substring starting at position `start` (0-indexed). If `length` is
provided, at most that many characters are returned; otherwise the rest of the
string from `start` is returned:

```text
logs | set attributes["prefix"] = substring(attributes["trace.id"], 0, 8)
```

```text
// extract everything after the first 4 characters
logs | set attributes["suffix"] = substring(attributes["code"], 4)
```

### `replace(string, from, to)`

Replaces all occurrences of `from` with `to` in `string`:

```text
logs | set body = replace(body, "\n", " ")
```

### `ltrim(string)`

Removes leading whitespace from a string:

```text
logs | set body = ltrim(body)
```

### `rtrim(string)`

Removes trailing whitespace from a string:

```text
logs | set body = rtrim(body)
```

### `regexp_capture(string, pattern, group)`

Extracts a capture group from a regex match. The `group` parameter specifies
which capture group to return (1-indexed):

```text
// extract the domain from a URL
logs | set attributes["url.domain"] = regexp_capture(
    attributes["url.full"],
    r"https?://([^/]+)/.*",
    1
)
```

### `regexp_substr(string, pattern [, start [, occurrence [, flags [, group]]]])`

Extracts a substring matching a regular expression pattern. All parameters
after `pattern` are optional:

- `start` (integer, default `1`) -- 1-based character position to begin
  searching
- `occurrence` (integer, default `1`) -- which occurrence of the pattern to
  return
- `flags` (string) -- regex modifier flags: `i` (case-insensitive),
  `m` (multi-line), `s` (dot-all)
- `group` (integer, default `0`) -- which capture group to return; `0` returns
  the full match

```text
// extract the first numeric sequence from a string
logs | set attributes["error.code"] = regexp_substr(attributes["error.message"], r"\d+")
```

```text
// extract the second word, case-insensitively
logs | set attributes["second.word"] = regexp_substr(body, r"\w+", 1, 2, "i")
```

## Hashing Functions

Hashing functions compute a hash of the input value. They are commonly used
with `encode` to produce human-readable output.

### `sha256(value)`

Computes the SHA-256 hash. Returns a binary value:

```text
logs | set attributes["user.id"] = encode(sha256(attributes["user.id"]), "hex")
```

### `sha512(value)`

Computes the SHA-512 hash. Returns a binary value:

```text
logs | set attributes["token"] = encode(sha512(attributes["token"]), "hex")
```

### `md5(value)`

Computes the MD5 hash. Returns a string:

```text
logs | set attributes["checksum"] = md5(body)
```

### `fnv(value)`

Computes the FNV (Fowler-Noll-Vo) hash. Returns an integer:

```text
logs | set attributes["bucket"] = fnv(attributes["user.id"])
```

### `murmur3(value)`

Computes the MurmurHash3 hash. Returns an integer:

```text
logs | set attributes["partition"] = murmur3(attributes["trace.id"])
```

### `xxh3(value)`

Computes the xxHash3 (64-bit) hash. Returns an integer:

```text
logs | set attributes["hash"] = xxh3(body)
```

### `xxh128(value)`

Computes the xxHash128 hash. Returns a binary value:

```text
logs | set attributes["hash"] = encode(xxh128(body), "hex")
```

## Encoding Functions

### `encode(value, encoding)`

Encodes a binary value as a string. Supported encodings include `"hex"` and
`"base64"`:

```text
logs | set attributes["user.id"] = encode(sha256(attributes["user.id"]), "hex")
```

This is most commonly used in combination with hashing functions to produce
readable output.

## Math Functions

### Arithmetic operators

The standard arithmetic operators `+`, `-`, `*`, `/`, and `%` (modulus) are
available for numeric expressions:

```text
logs | set attributes["duration.seconds"] = attributes["duration.ms"] / 1000
```

### `log10(value)`

Computes the base-10 logarithm:

```text
logs | set attributes["magnitude"] = log10(attributes["count"])
```

## Date/Time Functions

### `format_datetime(timestamp, format)`

Formats a timestamp value as a string using a format pattern:

```text
logs | set attributes["date"] = format_datetime(time_unix_nano, "%Y-%m-%d")
```

### Date/time literals

Timestamp literals use the `timestamp` prefix:

```text
logs | where time_unix_nano < timestamp"2026-06-01T00:00:00.0"
```

## UUID Functions

### `uuid()`

Generates a random UUID v4 string. Each row receives a unique value:

```text
logs | set attributes["request.id"] = uuid()
```

### `uuidv7()`

Generates a UUID v7 string (time-ordered). Each row receives a unique value:

```text
logs | set attributes["event.id"] = uuidv7()
```

## Null Handling

### `coalesce(a, b, ...)`

Returns the first non-null value from the argument list. Requires at least two
arguments:

```text
logs | set body = coalesce(body, event_name, "unknown")
```
