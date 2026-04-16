# Profiling

This section covers CPU and memory profiling for `df_engine`.

## CPU profiling (samply)

**Requirements**:

- [samply](https://github.com/mstange/samply)

**Installation**:

```cmd/pwsh/bash
cargo install --locked samply
```

**Build**:

```cmd/pwsh/bash
cargo build --profile profiling --workspace
```

**Run**:

```pwsh/bash
samply record ./target/profiling/df_engine --config ./configs/otap-noop.yaml
```

**Result**:

CPU profiling output will be automatically rendered on browser.

## Memory profiling (dhat)

**Build**:

```cmd/pwsh/bash
cargo build --profile profiling --no-default-features --features dhat-heap --workspace
```

> [!NOTE]
> In this command, all default features are disabled.
> Use specific flags to enable individual features.

**Run**:

```pwsh/bash
./target/profiling/df_engine --config ./configs/otap-noop.yaml
```

**Result**:

On graceful shutdown of `df_engine`, it will generate `dhat-heap.json` file
for memory profiling that needs to be rendered by uploading it to:
<https://nnethercote.github.io/dh_view/>.

> [!NOTE]
> `dhat` needs a clean shutdown to generate `dhat-heap.json` file.
