# Filelog checkpoint v1 fixtures

Except for `checkpoint-encoding-variants.bin`, these deterministic fixtures are
generated from the version 1 format document
by `../generate_fixtures.py`. The generator uses only Python standard-library
big-endian packing and SHA-256 plus an explicit reflected Castagnoli CRC-32C
implementation. It does not invoke or import the Rust codec.

Regenerate from this crate with:

```console
python3 tests/generate_fixtures.py
```

`expected-values.txt` records the published digest vectors. The binary set
covers empty and active checkpoint containers, a container with a WAL transaction,
snapshot lifecycle shapes, every operation,
both transaction classes, all quarantine-reset actions, absent optional
metadata and non-administrative removal fields, `keep_failed` preservation and
mutation, the minimum transaction body, the 4,096-operation progress boundary,
zero-delta finalization, path representations, frontier guards, and framing
profiles.

The mutated `keep_failed` fixture is intentionally structurally decodable for
future replay rejection. Its unequal carried epochs are not valid output from
the current version 1 producer.

`framing-profile-default.bin` and `framing_profile_default` retain their historical
names for the explicit 500 ms newline profile. They do not represent the current
receiver default, which disables idle flush. Their bytes and digest remain
unchanged, including the profile carried by existing snapshot/WAL fixtures.
`framing-profile-idle-disabled.bin` and `framing_profile_idle_disabled` separately
cover the zero-millisecond profile using the same independent Python generator.
The codec takes explicit profile inputs; it does not select receiver defaults.

## Pre-optimization snapshot compatibility

`checkpoint-encoding-variants.bin` was captured with `encode_checkpoint` at
commit `655fa30d64b4e278cb12f6e7ca7a4adfdda009fa`, before the bounded encoder
change. Keep it unchanged; it is an oracle for the previous encoder's bytes.
It uses generation 9 and checkpoint ID `encoding-variants`. Its ten records
cover empty, 16-byte, and 65535-byte fingerprints; unavailable, short, full-size,
and truncated paths; Unix and Windows locators/paths; clean and continuation
framing; Active, Quarantined, and RotatedFinalized states; and the maximum-size
record combining a Windows locator, continuation, quarantine, and large fields.
Existing Python-generated golden fixtures remain unchanged and continue to
provide independent format coverage, including the empty snapshot.
