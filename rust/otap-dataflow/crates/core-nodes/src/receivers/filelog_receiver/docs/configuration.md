# Filelog configuration

This module checks Filelog settings and prepares them for the receiver. It does
not open files or start reading logs.

See the [Phase 1 spec][settings] for all settings, defaults, and accepted values.

## What configuration prepares

`Config::from_json` reads the settings supplied by the engine. Unknown fields
and unsupported values are rejected. Errors identify the affected field. Byte
sizes and durations use the shared parsers, including values such as `1MiB`
and `5s`.

`Config::compile` checks the values and how they work together. It prepares:

- File include/exclude rules and the optional multiline matcher.
- The checkpoint ID and information needed to check recovery compatibility.
- Size and memory estimates.

The result separates `ValidatedInputs` from `PathPolicy`. Use `inputs()` and
`paths()` to borrow them, or `into_parts()` to move them to separate owners.
Startup can still use the settings, framing, and checkpoint inputs after sending
the path policy to the discovery worker.

## File patterns

Patterns use `globset` syntax. `*` and `?` match within one path component;
`**` supports matching across directories. Brackets and alternatives, such as
`[ab]` and `{app,worker}`, are supported.

Absolute patterns need no base directory. Relative patterns require an explicit
absolute base from the caller. Without a base, the caller passes an empty path
and relative patterns are rejected. There is no working-directory or checkpoint
root fallback.

Pattern text must be UTF-8. Unix filenames keep their original bytes, including
invalid UTF-8. Absolute patterns do not require a UTF-8 base. Unix glob escapes
are removed from literal roots; other root spelling is preserved.

Patterns containing `..` components or ending in a separator are rejected.
Empty or `.` components after a wildcard are also rejected.

The policy avoids directories that cannot contain a match. For example,
`/logs/*.log` does not need to scan child directories. This also works for
alternatives containing separators, such as `{a/b,c}`. After a matching `**`,
traversal can continue deeper, subject to discovery's depth limit.

## Excluding files

Exclusion wins over inclusion. Discovery checks both the path used to find a
file and its verified target, so an alias cannot bypass an exclusion.

`excludes_mapped` checks the remaining relative path under a verified exclusion
root. Its index must follow the order returned by `exclude_roots()`. Invalid
indices and non-relative suffixes are treated as excluded.

A prefix is not automatically excluded because a rule uses it. Only a matching
exclusion ending in `/**` allows the whole subtree to be skipped. Matching a
path does not establish its identity or permission to read it.

## Saved reading progress

An explicit `checkpoint.id` selects the checkpoint namespace. Otherwise, the ID
is derived from the configured pipeline-group, pipeline, and node IDs. Worker
placement and discovery settings do not affect it.

Changing the explicit checkpoint ID, or a logical ID used to derive it, selects
a different namespace. There is no automatic search for old checkpoints or
migration. Configuration and storage share the checkpoint crate's ID encoding,
compatibility digest, and size calculations. See the [checkpoint format][checkpoint]
for the exact rules.

## Fingerprint ambiguity

Fingerprint evidence is a bounded raw prefix after `ignored_header_bytes`, not a
unique file identifier. For a changed locator, a nonempty equal common prefix
can associate the candidate with retained Active state even when the file is
shorter than `fingerprint_bytes`. The candidate then follows
`identity.on_recovery_mismatch`, rather than `start_at` or `ignore_older_than`.
This intentionally favors replay of surviving copied bytes over skipping them.

The configured window is a maximum, not a minimum amount of stored evidence.
A file observed while tiny can leave one-byte evidence even with the default
1,000-byte window, until growth is observed and persisted.

Smaller windows, including the 16-byte minimum, increase false associations.
Independent files can share timestamps or headers; a larger window reduces but
does not eliminate that ambiguity. Under `beginning`, false association can
replay existing content; under `fail`, it can quarantine a new unrelated file.
No association transfers the old file's ID, epoch or committed offset.

## Memory checks

Configuration rejects arithmetic overflow and values above the limits in the
[resource models][resources]. These include the provisional 1 GiB identity
model and checkpoint file/recovery ceilings. Errors report which calculation
failed, why it failed, and the contributing settings.

Estimates cover file tracking, readers, framing, checkpoints, matching, and a
batch plus one carry-over record. Record-size checks include metadata and
attributes. The minimum partial-state estimate includes framing buffers and
unread bytes left from one read turn. Startup must add runtime reader state
before deciding how many unfinished records it can hold.

Snapshot admission includes older fingerprint windows within the existing
checkpoint recovery ceilings. Reducing `identity.fingerprint_bytes` alone does
not shrink that budget. Retained-record and reader estimates allow the format's
maximum fingerprint size; incompatible records keep their existing evidence.

These estimates do not reserve memory or guarantee total process memory use.
Startup must include the remaining costs without counting shared storage twice.

## Matching implementation

Globset translates syntax; the shared regex engine compiles and runs it.
Patterns are compiled before scanning and reused for later queries. Program
limits depend on pattern complexity. Lazy caches have a 2 MiB limit with an
equivalent fallback matcher. These are internal limits, not additional YAML settings.

Mapped exclusions share one indexed matcher and cache; directory-component
checks share another. Each path policy owns its working memory. It can move to
a worker, but is not shared concurrently. Unix queries borrow path bytes rather
than allocating joined paths.

`retained_bytes()` covers programs, metadata, roots, and matching buffers.
Allocation tests check initial storage separately from cache growth. Temporary
compilation memory and allocator overhead are separate costs. The
[matching code][matching] and [regex memory model][regex-memory] document the
implementation assumptions that need review when dependencies change.

## Startup and runtime responsibilities

Configuration validation checks settings and prepares matching rules.
Receiver startup performs the filesystem and runtime checks:

- Use the engine-provided state-root capability to open and recover checkpoints
  under the store's ownership rules.
- Resolve and verify file roots. Call `validate_root_lengths` with the scanner's
  capacity before constructing the scanner.
- Reject direct inclusion of the checkpoint namespace and prevent reading it
  through aliases, using verified file identities.
- Verify discovery support, available resources, and downstream completion
  requirements before reporting readiness.
- Include reader state, worker buffers, channels, leases, batches, and
  compilation memory in the receiver's memory budget.

During operation, the receiver checks deadline and counter arithmetic when
those values are used.

The path policy provides matching decisions. Discovery owns directory scanning
and symlink handling. The runtime owns scheduling.

Each supported platform requires its own path and filesystem qualification.

[settings]: ../../../../../../docs/filelog-receiver-phase1-spec.md#fields-defaults-and-variants
[checkpoint]: ../../../../../../docs/filelog-checkpoint-format.md
[resources]: ../../../../../../docs/filelog-receiver-phase1-conformance.md
[matching]: ../config/glob_program.rs
[regex-memory]: ../multiline_pattern/memory.rs
