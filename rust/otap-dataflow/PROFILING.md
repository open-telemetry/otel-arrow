# Profiling

This section covers CPU and memory profiling for `df_engine`.

**Requirements**:

- [samply](https://github.com/mstange/samply)

**Installation**:

```cmd/pwsh/bash
cargo install --locked samply
```

**Build**:

**Build for both CPU and memory profiling**:

```cmd/pwsh/bash
cargo build --profile profiling --no-default-features --features dhat-heap --workspace
```

> [!NOTE]
> In this command, all default features are disabled.
> Use specific flags to enable individual features.

**Build for only CPU profiling**:

```cmd/pwsh/bash
cargo build --profile profiling --workspace
```

**Run**:

**Run with both CPU and memory profiling enabled**:

```pwsh/bash
samply record ./target/profiling/df_engine --config ./configs/otap-noop.yaml
```

**Run with only memory profiling enabled**:

```pwsh/bash
./target/profiling/df_engine --config ./configs/otap-noop.yaml
```

**Result**:

On graceful shutdown of `df_engine`, it will generate `dhat-heap.json` file
for memory profiling that needs to be rendered by uploading it to:
<https://nnethercote.github.io/dh_view/>.

CPU profiling output will be automatically rendered on browser.

> [!NOTE]
> `dhat` needs a clean shutdown to generate `dhat-heap.json` file.
> In `df_engine` this can be done manually with Ctrl+C.
