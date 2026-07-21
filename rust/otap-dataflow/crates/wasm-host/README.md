# otap-df-wasm-host

WASM host-kernel runtime for OTAP dataflow processor plugins.

> **Status: experimental / unstable.** This crate is under active
> development. The WIT contract (`wit/plugin.wit`) and the host surface are
> **not stable** and are subject to breaking changes without notice while the
> WASM binary plugin system is being built out. It is gated behind an
> off-by-default cargo feature (see [Feature flag](#feature-flag)). Do not
> depend on any part of this crate outside the in-tree experimental slice.
>
> **Pre-release contract warning:** This crate and `wit/plugin.wit` are an
> internal experimental interface. **Do not ship production plugins or take
> external dependencies on this WIT package yet.** We expect breaking changes
> in upcoming phases (resource model, processor/effects contract, kernel
> semantics, and error/failure policies), and compatibility across revisions is
> not guaranteed.

This crate implements the initial slice of the WASM binary plugin system (see
[open-telemetry/otel-arrow#2973][parent] and [#3227][wit]): a thin,
end-to-end vertical slice that proves the host-kernel processor-plugin
mechanism works against the real `otap-dataflow` engine.

## Feature flag

The runtime is disabled by default. All wasmtime-backed functionality (the
generated bindings, native kernels, the `OtapPdata` bridge, and the
`wasm_processor` factory registration) lives behind the `wasm` cargo feature,
which is **off by default**:

```toml
otap-df-wasm-host = { workspace = true, features = ["wasm"] }
```

With the feature off, the crate compiles to an empty shell and pulls in no
wasmtime dependency. Enable `wasm` to build and register the processor.

## What it does

- Loads a `.wasm` component plugin at pipeline startup (compiled once per
  core; no compile/instantiate in the hot path).
- Runs it as a standard processor node registered through the engine's
  `ProcessorFactory` / `distributed_slice` pattern
  (URN `urn:otel:processor:wasm_processor`).
- Passes an opaque, **host-managed pdata resource** across the host-guest
  boundary. Bulk Arrow data never crosses the WASM boundary; the host owns
  the underlying `RecordBatch` data and runs kernels natively.
- Bridges `OtapPdata` <-> Arrow `RecordBatch` while preserving the pdata
  `Context` (Ack/Nack routing and transport headers).

## Host-kernel orchestration model

The guest issues OTel-semantic kernel commands over an opaque `pdata`
resource handle; the host executes them natively on Arrow arrays and
validates the reconstructed batch against OTAP schema invariants before
forwarding downstream.

The WIT contract (`wit/plugin.wit`) freezes only the tiny surface this
slice needs:

- `otel-kernels`: the `pdata` resource, `pdata-num-rows`,
  `filter-by-attribute-eq`, and the `attr-scope` enum.
- `processor`: `process(data) -> option<pdata>` (return `none` to drop).
- the `kernel-processor` world.

Current experimental behavior is intentionally narrow:

- `filter-by-attribute-eq` currently supports only `record` scope.
- `resource` and `scope` are rejected until they are implemented.
- Invalid pdata handles and invalid filter operations are rejected rather than
  silently treated as no-ops.
- Filtering currently targets the root record batch only; child-batch
  relationship filtering is deferred.

## Configuration

```yaml
nodes:
  my-filter:
    type: processor:wasm_processor
    config:
      wasm_path: "/plugins/severity_filter.wasm"
```

## Reference guest plugin

`plugins/severity-filter/` is a `no_std`, `wasm32-wasip2` reference plugin
that filters log records where `severity_text == "ERROR"`. The WASM binary is intentionally
excluded from the Cargo workspace and built on demand by the
integration test.

## Deferred to later phases

The full kernel vocabulary, regex/hash/redact/truncate kernels, the escape
hatches, the OPL path, an AOT module cache, epoch-interruption resource
limits, a polished guest SDK, and the exporter/receiver/extension worlds are
all out of scope for this initial implementation.

[parent]: https://github.com/open-telemetry/otel-arrow/issues/2973
[wit]: https://github.com/open-telemetry/otel-arrow/issues/3227
