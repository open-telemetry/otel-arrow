# Filelog checkpoint v1 fixtures

These deterministic fixtures are generated from the version 1 format document
by `../generate_fixtures.py`. The generator uses only Python standard-library
big-endian packing and SHA-256 plus an explicit reflected Castagnoli CRC-32C
implementation. It does not invoke or import the Rust codec.

Regenerate from this crate with:

```console
python3 tests/generate_fixtures.py
```

`expected-values.txt` records the published digest vectors. The binary set
covers `CURRENT`, snapshot lifecycle shapes, the WAL header, every operation,
both transaction classes, all quarantine-reset actions, absent optional
metadata and non-administrative removal fields, `keep_failed` preservation and
mutation, the minimum transaction body, the 4,096-operation progress boundary,
zero-delta finalization, path representations, frontier guards, and framing
profiles.

The mutated `keep_failed` fixture is intentionally structurally decodable for
future replay rejection. Its unequal carried epochs are not valid output from
the current version 1 producer.
