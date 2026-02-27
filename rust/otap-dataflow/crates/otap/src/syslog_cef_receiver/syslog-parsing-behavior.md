# Syslog Parsing Behavior

## Format Detection Order

The top-level `parse()` function in
`crates/otap/src/syslog_cef_receiver/parser/mod.rs`
tries formats in this order:

<!-- markdownlint-disable MD038 -->

1. **Pure CEF** — input starts with `CEF:`
2. **RFC 5424** — requires `<PRI>VERSION ` structure
3. **RFC 3164** — very lenient fallback (accepts almost anything non-empty)
4. **Error** — only if all three fail (practically only on empty input)

<!-- markdownlint-enable MD038 -->

---

## Positive Cases (Successful Parsing)

<!-- markdownlint-disable MD013 -->

| # | Input Example | Detected As | Parsed Fields | Body Set? | Notes |
|---|---|---|---|---|---|
| 1 | `<34>1 2003-10-11T22:14:15.003Z host app - ID47 - msg` | `Rfc5424` | priority, version, timestamp, hostname, app_name, msg_id, message | No (fully parsed) | Standard RFC 5424 |
| 2 | `<34>1 - - - - - - msg` | `Rfc5424` | priority, version, message | No | All optional fields nil (`-`) |
| 3 | `<34>1 - - - - - [sd@123 k="v"] msg` | `Rfc5424` | priority, version, structured_data, message | No | With structured data |
| 4 | `<34>1 - - - - - [sd1@1 k="v"][sd2@2 k="v"] msg` | `Rfc5424` | priority, version, structured_data, message | No | Multiple SD-ELEMENTs |
| 5 | `<34>1 ... - <BOM>msg` | `Rfc5424` | priority, version, message (BOM stripped) | No | UTF-8 BOM (`0xEF 0xBB 0xBF`) removed from message start |
| 6 | `<34>Oct 11 22:14:15 host su: content` | `Rfc3164` | priority, timestamp, hostname, tag, app_name, content | No | Standard RFC 3164 |
| 7 | `<34>Oct 11 22:14:15 host sshd[5678]: content` | `Rfc3164` | priority, timestamp, hostname, tag, app_name, proc_id, content | No | TAG with numeric PID extracted |
| 8 | `<34>hostname tag: message` | `Rfc3164` | priority, hostname, tag, app_name, content | No | No timestamp |
| 9 | `<34>This is just content` | `Rfc3164` | priority, content | No | No colon → entire remainder is content |
| 10 | ```CEF:0\|Vendor\|Product\|1.0\|100\|name\|10\|src=1.2.3.4``` | `Cef` | All 7 CEF headers + extensions | No | Pure CEF, no syslog wrapper |
| 11 | ```CEF:0\|V\|P\|1.0\|100\|name\|10\|``` | `Cef` | All 7 CEF headers, empty extensions | No | Trailing pipe, no extensions |
| 12 | ```CEF:0\|V\|P\|1.0\|100\|name\|10``` | `Cef` | All 7 CEF headers, no extensions | No | No trailing pipe |
| 13 | ```<134>1 2024-... host CEF - - CEF:0\|V\|P\|1.0\|100\|name\|10\|k=v``` | `CefWithRfc5424` | RFC 5424 fields + CEF fields + extensions | No | CEF embedded in RFC 5424 message |
| 14 | ```<34>Oct 11 22:14:15 fw CEF: CEF:0\|V\|P\|1.0\|100\|name\|10\|``` | `CefWithRfc3164` | RFC 3164 fields + CEF fields | No | CEF embedded in RFC 3164 content |
| 15 | ```<34>Oct 11 22:14:15 host CEF:0\|V\|P\|1.0\|100\|name\|10\|``` | `CefWithRfc3164` | RFC 3164 fields + CEF fields | No | Special case: TAG=`CEF`, parser reconstructs full CEF from input |

<!-- markdownlint-enable MD013 -->

---

## Negative / Edge Cases

### Cases That Return Errors (Message Rejected)

<!-- markdownlint-disable MD013 -->

| # | Input Example | Error | Notes |
|---|---|---|---|
| 1 | `""` (empty) | `UnknownFormat` | All three sub-parsers fail with `EmptyInput`; `parse()` returns `UnknownFormat`. This is the **only** input that `parse()` rejects (RFC 3164 accepts everything else). |

### Cases That Parse Successfully but with Degraded/Partial Results

| # | Input Example | Detected As | What Happens | Body Set? | `is_fully_parsed()` |
|---|---|---|---|---|---|
| 2 | `CEF:` (nothing after prefix) | `Rfc3164` | CEF parser returns `EmptyCEFContent`; RFC 5424 fails (no `<`); RFC 3164 succeeds with priority=None, tag=`CEF`, content=empty | **Yes** | `false` |
| 3 | `Use the BFG!` (no PRI, no structure) | `Rfc3164` | No priority; entire input becomes `content` | **Yes** (original input) | `false` |
| 4 | `<00>Test message` (leading-zero PRI) | `Rfc3164` | PRI invalid → priority=None; entire input (`<00>Test message`) becomes `content` | **Yes** | `false` |
| 5 | `<999Test message` (unclosed PRI) | `Rfc3164` | PRI parse fails → priority=None; entire input is `content` | **Yes** | `false` |
| 6 | `<abc> Test message` (non-numeric PRI) | `Rfc3164` | PRI parse fails → priority=None; entire input is `content` | **Yes** | `false` |
| 7 | `<> Test message` (empty PRI) | `Rfc3164` | PRI parse fails → priority=None; entire input is `content` | **Yes** | `false` |
| 8 | `<192>1 - - - - - - msg` (PRI > 191) | `Rfc3164` | RFC 5424 PRI validation fails; RFC 3164 also treats PRI as invalid → content-only | **Yes** | `false` |
| 9 | `Oct 11 22:14:15 host su: msg` (no PRI, has structure) | `Rfc3164` | No priority but timestamp/hostname/tag/content parsed | **Yes** | `false` |
| 10 | ```Oct 11 22:14:15 host CEF:0\|V\|P\|1.0\|100\|name\|10\|k=v``` (CEF+3164 no PRI) | `CefWithRfc3164` | Both 3164 & CEF parsed, but no priority | **Yes** | `false` |
<!-- markdownlint-disable-next-line MD038 -->
| 11 | `<34>1 - - - - - [id@123 key="value" ` (unclosed SD) | `Rfc5424` | Unclosed structured data → entire remainder captured as SD; message=None | No | `true` |
| 12 | `<34>1 - - - - - [ Message` (single open bracket) | `Rfc5424` | Unclosed bracket → everything after `[` treated as SD; message=None | No | `true` |
| 13 | `<34>Oct 11 22:14:15 host app[worker-1]: msg` (non-numeric PID) | `Rfc3164` | tag=`app[worker-1]`, app_name=`app`, proc_id=**None** (non-numeric rejected) | No | `true` |
| 14 | `<34>hostname app[]: message` (empty brackets) | `Rfc3164` | app_name=`app`, proc_id=**None** (empty brackets) | No | `true` |
| 15 | `<34>hostname app[123: message` (unclosed bracket) | `Rfc3164` | Entire `app[123` treated as app_name; proc_id=None | No | `true` |
| 16 | `<34>1` (priority + version, no space) | Fails RFC 5424 → `Rfc3164` | RFC 5424 requires space after version; falls to RFC 3164, content=`1` | No | `true` |
| 17 | `<34>` (priority only) | Fails RFC 5424 → `Rfc3164` | RFC 5424 `InvalidVersion`; RFC 3164 succeeds with priority, empty content | No | `true` |

### CEF-Specific Negative Cases

| # | Input Example | Error / Outcome | Notes |
|---|---|---|---|
| 18 | ```CEF:0\|vendor\|product\|version\|id``` (only 4 pipes) | `InvalidCef` → falls to RFC 3164 | Fewer than 7 required fields |
| 19 | ```CEF:0\|vendor\|product\|version\|id\|name``` (only 5 pipes) | `InvalidCef` → falls to RFC 3164 | Missing severity field |
| 20 | ```CEF:2.0\|V\|P\|1.0\|100\|name\|10\|``` | `InvalidCef` → falls to RFC 3164 | CEF version must be 0 or 1 |
| 21 | ```CEF:0\|\|\|\|\|\|\|``` (all empty fields) | **Succeeds** as `Cef` | All 7 header fields present but empty; valid per parser |
| 22 | ```CEF:0\|V\|P\|1.0\|100\|name\|10\|=``` (ext: only `=` sign) | Succeeds, 0 extensions | Empty key skipped gracefully |
| 23 | ```CEF:0\|V\|P\|1.0\|100\|name\|10\|===value``` | Succeeds, 0 extensions | Empty key skipped |
| 24 | ```CEF:0\|V\|P\|1.0\|100\|name\|10\|key=value\\``` (trailing backslash) | Succeeds, 1 extension | Trailing `\` preserved as-is in value |
| 25 | ```CEF:0\|V\|P\|1.0\|100\|name\\|10\|``` (escaped pipe in header) | Succeeds, but pipe becomes part of `name` field | `name` = ```name\|10```, severity = empty |

<!-- markdownlint-enable MD013 -->

---

## Priority Parsing Edge Cases

<!-- markdownlint-disable MD013 -->

| Input PRI | Valid? | facility | severity | Notes |
|---|---|---|---|---|
| `<0>` | Yes | 0 | 0 | Minimum valid |
| `<191>` | Yes | 23 | 7 | Maximum valid |
| `<192>` | No | — | — | Exceeds max (facility 24 invalid) |
| `<00>` | No | — | — | Leading zero makes PRI "unidentifiable" per RFC 3164 §4.3.3 |
| `<01>` | No | — | — | Leading zero |
| `<abc>` | No | — | — | Non-numeric |
| `<>` | No | — | — | Empty value, `end < 2` check fails |
| `<1234>` | No | — | — | Too many digits (`end > 4`) |

<!-- markdownlint-enable MD013 -->

---

## Syslog Severity → OTel Severity Mapping

<!-- markdownlint-disable MD013 -->

| Syslog Severity | Syslog Name | OTel Severity Number | OTel Severity Text |
|---|---|---|---|
| 0 | Emergency | 21 | `FATAL` |
| 1 | Alert | 19 | `ERROR3` |
| 2 | Critical | 18 | `ERROR2` |
| 3 | Error | 17 | `ERROR` |
| 4 | Warning | 13 | `WARN` |
| 5 | Notice | 10 | `INFO2` |
| 6 | Informational | 9 | `INFO` |
| 7 | Debug | 5 | `DEBUG` |

<!-- markdownlint-enable MD013 -->

---

## Key Behavioral Summary

<!-- markdownlint-disable MD038 -->

- **RFC 3164 is the ultimate fallback** — it accepts any
  non-empty input, even completely unstructured text. When
  PRI is missing/invalid, priority is `None`,
  `is_fully_parsed()` returns `false`, and the log **body**
  is set to the raw input for debugging.
- **Empty input is the only true rejection** —
  `parse()` returns `Err` only for empty input (since
  RFC 3164 catches everything else).
- **CEF parsing failures are non-fatal** — if CEF header
  parsing fails (wrong version, too few pipes), the input
  falls through to RFC 5424 / RFC 3164.
- **RFC 5424 is strict on PRI+VERSION** — requires valid
  `<PRI>VERSION ` structure; any deviation causes it to
  fall through to RFC 3164.
- **Structured data is lenient** — unclosed brackets are
  captured as-is rather than causing errors; escaped
  characters (`\"`, `\]`, `\\`) inside quoted values
  are handled correctly.

<!-- markdownlint-enable MD038 -->
