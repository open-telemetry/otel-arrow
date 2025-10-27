# Geneva Exporter Architecture

## Location and Purpose

This Geneva exporter is located in `rust/experimental/gigwarm_exporter/` and will **remain in the experimental directory permanently**.

### Why Experimental?

1. **Microsoft-Internal:** This exporter is designed exclusively for Microsoft products to send data to Geneva monitoring backend
2. **Clear Boundaries:** Keeping it in `experimental/` signals it's not meant for general use
3. **Independent Evolution:** Can evolve independently from the main OTAP workspace
4. **Relaxed Constraints:** Can use relaxed lints and experimental patterns without affecting production code

## Workspace Structure

```
otel-arrow/rust/
├── otap-dataflow/              # Main OTAP dataflow workspace
│   ├── Cargo.toml             # Workspace members: ["crates/*", ...]
│   ├── crates/
│   │   └── otap/
│   │       └── src/
│   │           └── geneva_exporter.rs  # ← Working Geneva exporter (uses geneva-uploader)
│   └── src/
│       └── main.rs            # df_engine binary
│
└── experimental/               # Experimental crates (NOT in workspace)
    └── gigwarm_exporter/      # ← This scaffold (permanent location)
        ├── Cargo.toml
        ├── src/lib.rs
        └── README.md
```

## Two Geneva Exporters?

There are currently **two** Geneva-related exporters in the codebase:

### 1. Working Geneva Exporter (Production)
- **Location:** `otap-dataflow/crates/otap/src/geneva_exporter.rs`
- **Status:** Fully functional, production-ready
- **Dependencies:** Uses `geneva-uploader` crate
- **Integration:** Built into `df_engine` binary via workspace
- **Purpose:** Powers the OTAP dataflow Geneva export functionality

### 2. This Scaffold (Experimental)
- **Location:** `experimental/gigwarm_exporter/` (this crate)
- **Status:** No-op scaffold for incremental PR development
- **Dependencies:** Zero (intentional)
- **Integration:** Independent crate, not in workspace
- **Purpose:** Demonstrate incremental API development for otel-arrow repo

## Build and Test

### This Experimental Crate

```bash
# Build independently
cd rust/experimental/gigwarm_exporter
cargo build
cargo test
cargo clippy
```

This crate is **NOT** built when you build the main workspace:

```bash
cd rust/otap-dataflow
cargo build --workspace  # ← Does NOT include experimental/gigwarm_exporter
```

### Main OTAP Workspace (includes working Geneva exporter)

```bash
cd rust/otap-dataflow
cargo build --bin df_engine  # ← Includes otap/src/geneva_exporter.rs
```

## Development Roadmap

This scaffold will evolve through incremental PRs:

1. **PR 1 (Current):** Scaffold with no-op implementations
2. **PR 2:** Add configuration and validation
3. **PR 3:** Add Arrow RecordBatch processing
4. **PR 4:** Add Geneva Bond encoding
5. **PR 5:** Add HTTP upload logic
6. **PR 6:** Integration tests

Each PR keeps the crate in `experimental/` - this is the permanent location.

## Design Decisions

### Why Not Merge with Production Exporter?

The production exporter (`otap/src/geneva_exporter.rs`) and this experimental scaffold serve different purposes:

- **Production:** Integrated into OTAP dataflow engine, uses existing `geneva-uploader` crate
- **Experimental:** Demonstrates incremental development, shows API evolution, may experiment with different approaches

### Why Not Add to Workspace?

1. **Strict Lints:** Main workspace has `deny` lints that would fail on scaffold code
2. **Clear Separation:** Experimental code should be obviously separate
3. **Independent Versioning:** Can evolve at its own pace
4. **No Accidental Usage:** Can't accidentally import from non-workspace crate

### Why Keep It?

Even though there's a working exporter, this scaffold serves as:
- **API Documentation:** Shows how Geneva exporter API should look
- **Incremental Example:** Demonstrates best practices for PR submission
- **Experimentation:** Can try new approaches without affecting production

## Integration Path (Future)

If this experimental exporter is eventually promoted:

**Option A: Replace existing**
```bash
# Move experimental to replace production
mv experimental/gigwarm_exporter otap-dataflow/crates/geneva-exporter
# Update Cargo.toml workspace members
# Remove old geneva_exporter.rs
```

**Option B: Feature-gate both**
```toml
[features]
geneva-v1 = []  # Original exporter
geneva-v2 = []  # New experimental exporter
```

**Option C: Keep separate**
- Production code uses `otap/src/geneva_exporter.rs`
- Experimental stays in `experimental/gigwarm_exporter/`
- Both can coexist for different use cases

## Summary

✅ **Permanent Location:** `rust/experimental/gigwarm_exporter/`
✅ **Not in Workspace:** Intentional design decision
✅ **Independent Build:** `cargo build` from its directory
✅ **Microsoft-Internal:** Clearly marked as experimental
✅ **No Conflicts:** Doesn't interfere with production exporter

This architecture allows clean separation while enabling incremental development and experimentation.
