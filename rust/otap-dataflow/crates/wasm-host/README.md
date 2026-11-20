# otel-arrow-dfe-wasm-host

WASM host-kernel runtime for OTAP dataflow processor plugins.

> **Status: experimental / unstable.** This crate is under active
> development. The WIT contract (`../wasm-guest-sdk/wit/plugin.wit`) and the
> host surface are
> **not stable** and are subject to breaking changes without notice while the
> WASM binary plugin system is being built out. It is gated behind an
> off-by-default cargo feature (see [Feature flag](#feature-flag)). Do not
> depend on any part of this crate outside the in-tree experimental slice.
>
> **Do not ship production plugins or take external dependencies on this WIT
> package yet.** Before WIT package version `1.0.0`, backward compatibility is
> not guaranteed. Host upgrades may require updating plugin source and rebuilding
> binaries against the matching SDK.

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
otel-arrow-dfe-wasm-host = { workspace = true, features = ["wasm"] }
```

With the feature off, the crate compiles to an empty shell and pulls in no
wasmtime dependency. Enable `wasm` to build and register the processor. This
experimental feature requires Rust 1.96 or newer due to its wasmtime
dependency.

## What it does

- Loads a `.wasm` component plugin at pipeline startup (compiled once per
  core; no compile/instantiate in the hot path).
- Runs it as a standard processor node registered through the engine's
  `ProcessorFactory` / `distributed_slice` pattern
  (URN `urn:otel:processor:wasm_processor`).
- Passes an opaque, **host-managed pdata resource** across the host-guest
  boundary. Bulk Arrow data never crosses the WASM boundary; the host owns
  OTAP records and runs kernels natively.
- Materializes `OtapPdata` into native records through pipeline codecs while
  retaining original delivery ownership on the host. Multiple outputs preserve
  metadata without duplicating upstream Ack/Nack ownership.

## Host-kernel orchestration model

The guest issues OTel-semantic kernel commands over an opaque `pdata`
resource handle; the host executes them natively on Arrow arrays and
forwards returned OTAP records without additional bridge-level schema
re-validation.

The WIT contract in `../wasm-guest-sdk/wit/plugin.wit` freezes only the tiny
surface this slice needs:

- `otel-kernels`: the `pdata` resource, `pdata-num-rows`, `clone-pdata`,
  `filter-by-attribute-eq`, and the `attr-scope` enum.
- `processor`: `process(data) -> list<output>`, where each output owns pdata
  and optionally names a port. An empty list intentionally drops the input.
- `host-services`: a capability-limited, host-provided import giving the
  guest an ABI version (`host-abi-version`), read-only config access
  (`get-config`), read-only output discovery (`get-output-info`),
  structured logging (`log`), and a bounded counter
  (`counter-add`).
- The host also links only the WASI 0.3 CLI and clocks interfaces needed by
  Rust `std`. CLI state is inert (empty arguments/environment, no cwd, and
  empty stdio). Filesystem, sockets, random, and all other WASI capabilities
  remain unavailable. Clock reads are available in every guest entry point,
  but future clock waits are rejected immediately in every phase.
- `lifecycle`: a guest-exported interface with `initialize` (called exactly
  once per plugin instance, after instantiation and before any `process`
  call).
- the `kernel-processor` world (`import otel-kernels; import host-services;
  export processor; export lifecycle;`).

Current experimental behavior is intentionally narrow:

- `filter-by-attribute-eq` currently supports only `record` scope.
- `resource` and `scope` are rejected until they are implemented.
- Invalid pdata handles and invalid filter operations are rejected rather than
  silently treated as no-ops.
- Record-scope filtering preserves OTAP parent/child relationships by applying
  the same selection to root and child batches.
- The processor reports minimal telemetry on `CollectTelemetry` for guest
  process calls, guest process errors, guest-driven drops, kernel invocation
  counters, per-signal `records_in`/`records_out`, and guest host-service
  activity (`counter-add` accepted/rejected, rate-limiter rejections, log
  truncations). The same tick emits the guest's cumulative per-name counter
  totals.
- The guest fetches its own config from within `initialize` by calling
  `host-services.get-config`, rather than receiving it as a function
  parameter, so the "host services" import remains the single mechanism for
  all guest/host interaction beyond raw pdata kernels. It is served only the
  plugin-owned `config` sub-object; host-internal fields such as `wasm_path`
  are not exposed to the guest.
- If a guest's `initialize` call returns an `init-error` (or traps), the
  `wasm_processor` factory fails at pipeline *construction* time with a
  config error -- misconfigured plugins never reach the `process` hot path.
- Guest `process` calls are awaited directly by the engine's existing async
  processor method. WASI clock reads are available, but future waits such as
  `std::thread::sleep` trap instead of suspending a node indefinitely.
- Guest `log` records, and the per-name counter totals, are emitted under the
  same component scope as native `wasm_processor` telemetry and are stamped
  by the host with the emitting pipeline node name (a `node` attribute).
  Guests cannot forge or suppress that attribution, so guest telemetry is
  always traceable to the specific plugin instance that produced it.

### Resource bounds and failure behavior

Every plugin instance runs under the following bounds. They are deliberately blunt;
see "Deferred to later phases" for what a real limits design still owes.

- **Fuel.** Each `initialize`/`process` call is granted a fixed
  Wasmtime fuel budget, so a runaway guest traps instead of hanging the
  pipeline thread.
- **Guest-call deadline.** Each `initialize`/`process` call has a provisional
  30-second host-owned deadline, including time suspended on component-model
  waitables. Initialization timeout fails pipeline construction; processing
  timeout is terminal for the instance and fails the node. The timer advances
  independently of Tokio during synchronous initialization. It cannot preempt
  synchronous native host kernels and is not a hard bound on processing latency.
- **Linear memory.** Each instance has a 64 MiB guest-memory cap.
- **String copies.** Canonical-ABI lifting has a 16 KiB encoded-input budget
  per lift, checked before copying strings into host memory. This covers
  imported arguments and exported results; bulk telemetry stays host-side.
  Log messages, counter names, kernel strings, config clones, and output-info
  copies additionally share a 2 MiB byte allowance per guest entry and a
  lifetime byte bucket
  (1 MiB/second, 2 MiB burst). Both aggregate limits charge UTF-8 bytes,
  including arguments to calls that are later ignored. Config is charged
  before cloning; incoming arguments are charged after lifting, so the
  failing call can copy at most one additional bounded argument set (up to
  32 KiB after encoding conversion). Output-info strings and list metadata are
  charged before copying. Exceeding either copy limit traps.
  The per-entry allowance does not refill while a call is running.
- **Host-service call rate.** `get-config`, `get-output-info`, `log`, and
  `counter-add` share a token-bucket limiter scoped to the instance's lifetime
  (sustained rate plus a burst allowance). Throttled telemetry calls within the
  copy budgets are silent no-ops; discovery returns an explicit `throttled`
  error.
- **Resource handles.** The host-managed component resource table is capped at
  10,000 entries during a guest call, and must be empty after all returned
  pdata are reclaimed. A guest that retains any host resource across
  `process` calls fails the node, so complete host-side batches cannot
  accumulate across messages.
- **Kernel invocations.** Each `initialize`/`process` call has a
  separate cap on native `otel-kernels` invocations. This is required because
  fuel accounts for guest Wasm instructions, not time spent inside native
  host imports; exceeding the cap traps the guest.
- **Output and completion counts.** `max_outputs_per_call` defaults to 64
  (allowed range 1..=256); `max_inflight_inputs` defaults to 128
  (allowed range 1..=1024). Zero is not an unlimited setting. For multi-output
  results with completion subscribers, the host reserves output correlation
  capacity before sending and pauses pdata admission at
  capacity while continuing to process completion controls. The 16 KiB
  canonical lifting budget also applies before allocating returned lists and
  port strings. Counts do not bound total native Arrow bytes or kernel latency.

Where the host silently drops a guest request, it always counts it, so
"the plugin is quiet" is distinguishable from "the plugin is being
throttled":

- Name too long: call ignored; increments
  `guest_counter_add_rejected_name_len`.
- Too many names: call ignored; increments
  `guest_counter_add_rejected_cardinality`.
- No tokens: call ignored, and `get-config` returns `""`; increments
  `guest_host_service_calls_rejected`.
- Log too long: message truncated and emitted; increments
  `guest_log_message_truncated`.
- Kernel call cap reached: guest call traps; increments
  `guest_process_errors`.

Copy-limit traps fail construction during `initialize`, or fail the node
during `process` and increment `guest_process_errors`. Host-services ABI
version 2 introduces these limits; plugins checking the ABI must accept
version 2 and keep control strings and aggregate copy work within the bounds.

Counter values accumulate with saturation, so a guest cannot overflow or wrap
host telemetry. `guest.counter.add.value` is reported as a cumulative total
for the plugin instance, saturating at `u64::MAX` across reporting intervals.

**Traps are terminal.** Wasmtime marks an instance permanently unusable after
any trap, and this host does not re-instantiate. A trap during `process`
(fuel exhaustion, a guest panic, or a kernel contract violation) fails that
call, and the engine terminates a node whose `process` returns an error. The
host records the instance as poisoned. Plugins should treat traps as fatal
rather than as a per-batch error channel.

Host-detected output cleanup failures and retained-resource violations also
poison the instance, even when Wasmtime itself has not trapped. Later calls
are rejected before inserting pdata or resetting execution budgets.

Correspondingly, host kernel implementations return traps rather than
panicking on guest-controlled input: the bindings are generated with trappable
imports so an unsupported `attr-scope`, an absent attribute key, or a stale
resource handle fails only that plugin, instead of aborting the collector
process.

Future clock waits trap immediately in `initialize` and `process`. This keeps
clock reads available for Rust `std` without introducing a guest-controlled
suspension that requires host cancellation support.

There is currently no guest finalization export. The engine's processor API
does not expose a post-drain lifecycle point after the last `process` call, so
the experimental WIT contract defers guest teardown rather than approximating
it through `Drop`.

## Enabling in `df_engine`

The shipping `df_engine` binary (`rust/otap-dataflow`) does not currently link
this crate by default. Opt in with the top-level `wasm` cargo feature, which
force-links `otel-arrow-dfe-wasm-host` (registering the `wasm_processor` factory)
and pulls in wasmtime:

```sh
cargo build -p otel-arrow-dfe --features wasm
```

Without `--features wasm`, `df_engine` compiles with no wasmtime dependency
and rejects any pipeline config referencing `processor:wasm_processor` with
`Unknown processor plugin urn:otel:processor:wasm_processor`.

## Configuration

```yaml
nodes:
  my-filter:
    type: processor:wasm_processor
    config:
      wasm_path: "/plugins/severity_filter.wasm"
      max_outputs_per_call: 64
      max_inflight_inputs: 128
      # Optional freeform config. This sub-object -- and only this
      # sub-object -- is serialized to JSON and served to the guest by
      # `host-services.get-config`.
      config:
        some_plugin_setting: true
```

## Multiple outputs and delivery

The SDK uses `Vec<Output>` with `Output { port: Option<String>, data: Pdata }`.
`None` selects the configured default port or the single connected port;
`Some(name)` must identify a connected port. Repeated destinations are allowed.
Use the ordinary node `outputs` and `default_output` configuration to wire
routes; they are not part of the plugin-owned `config` object.

The host validates all handles and destinations before dispatch. Output limit
violations, invalid handles, and unknown or ambiguous ports fail the node.
`clone-pdata` shares Arrow buffers through independently owned handles;
filtering a clone leaves the original unchanged.

`host-services.get-output-info` returns sorted connected ports, the resolved
default, and `max_outputs_per_call`. Declared but unconnected ports are excluded.
The snapshot is fixed before the first `process` call and can be cached. Discovery
returns `unavailable` during instantiation and initialization, and `throttled`
when the shared call-rate budget is exhausted. Copies charge the shared byte
budgets.

Dispatch follows returned order using normal engine sends and backpressure.
It is not transactional: a channel may close after an earlier output was sent.
As with other routing processors, a closed output channel is a node error;
remaining outputs are not sent. Input without a root record batch is Acked
without invoking the guest.

A single output reuses the input's delivery context, so its completion flows
directly upstream and consumes no tracking capacity. For multiple outputs, the
host retains one original delivery context. All successful outputs produce
one upstream Ack; any failure produces one Nack after all outputs settle.
The Nack is retryable only when every output failed transiently. Mixed success
and failure, or any permanent failure, makes the Nack permanent to avoid
automatically replaying successful deliveries.
Original input bytes/records are retained only when `RETURN_DATA` requests them.
Decode failures still recover the exact original input.

`records_out` counts rows produced by the guest across all outputs, including
intentional duplicates, before routing or delivery. Engine metrics measure the
subsequent sends and completions. Upstream flow-compute time is divided equally
among outputs, with any remainder assigned in list order.

The guest can retain bounded local state between calls, but not host pdata.
Completion handling is host-owned; there are no guest completion or shutdown
callbacks. Closing upstream input can stop completion handling before outstanding
outputs settle, including single-output calls (#2477; receiver-side work: #3849).
Cancellation and terminal failure can also prevent completion; teardown releases
resources without sending Ack/Nack.

When enabled, flow `dropped.items` reports the net item-count reduction across
the guest's returned outputs. An empty list drops every input item; duplicated
fanout rows can mask filtering in other outputs. `flow.compute.duration` measures
engine send-boundary timing, not synchronous `processor.compute.duration`.

The host supports WIT package `otel:otap-dataflow-plugin@0.2.0` and host-services
behavior version 2.

## Reference guest plugin

`plugins/severity-filter/` is a `std`, `wasm32-wasip3` reference plugin built
with Rust nightly that filters log records where `severity_text == "ERROR"`.
The WASM binary is intentionally excluded from the Cargo workspace and built
on demand by the integration test. It also demonstrates the
`lifecycle` and `host-services` contract through the in-repository
`wasm-guest-sdk` crate. The SDK owns guest binding generation and the current
host ABI v2 check, so plugin code does not depend directly on generated WIT
module paths or `wit_bindgen`.

The reference plugin intentionally contains no synthetic failure switches or
resource-abuse paths, so it remains a small example of normal plugin code.

## Test-only guest fixture

`plugins/test-plugin/` is a separate guest used only by host tests. It contains
deliberately abnormal behavior selected through config flags: returned
initialization errors, panics, fuel-exhausting loops, oversized allocations and
strings, copy floods, and clock waits during initialization and processing.
Keeping those paths in a test fixture prevents test requirements from turning
the reference plugin into an example that real plugin authors should not follow.

### Building a guest plugin

The guest depends on the SDK using a local Cargo path dependency and is built
for `wasm32-wasip3` with Rust nightly:

```bash
rustup target add --toolchain nightly wasm32-wasip3
cd crates/wasm-host/plugins/severity-filter
rustup run nightly cargo build --release --target wasm32-wasip3
```

Building the host with the experimental `wasm` feature requires Rust 1.96 or
newer because of Wasmtime 49. This feature-specific requirement does not affect
the workspace's default build, where the optional Wasmtime dependencies are
disabled.

`wasm32-wasip1` (WASI 0.1) predates the component model, so a `wasip1` binary
is a plain module, not a component, and cannot satisfy the SDK WIT package's
worlds at all.

Phase 1b guest plugins use Rust `std`. The host deliberately links only
`wasi:cli@0.3.0` and `wasi:clocks@0.3.0` in addition to the interfaces
declared by the `kernel-processor` world. The
`guest_imports_only_the_sandboxed_interfaces` integration test enforces that
allowlist, so importing filesystem, sockets, random, or any other ambient WASI
capability fails both the test and host instantiation. Check a plugin by hand
with `wasm-tools component wit <plugin>.wasm | grep import`.

## Deferred to later phases

The full kernel vocabulary, regex/hash/redact/truncate kernels, the escape
hatches, the OPL path, an AOT module cache, and the
exporter/receiver/extension worlds are all out of scope for this initial
implementation.

Also deferred, and more pressing:

- **A real resource-limit design.** The fuel and memory constants here are
  unprofiled placeholders, are not configurable per plugin or pipeline, and
  do not bound native-kernel latency. Follow-on work must account for batch
  sizes and cumulative native work, and provide cooperative kernel execution
  where needed. Epoch interruption only interrupts guest Wasm execution;
  it is not a backstop for synchronous host work.
- **A trap recovery policy.** Today any trap is terminal for the node. Whether
  a plugin should instead be re-instantiated, restarted with backoff, or
  bypassed is an open question.
- **Deterministic clocks.** The linked WASI clocks currently expose real wall
  and monotonic time. A later determinism-policy change will replace them with
  frozen host clocks.
- **A guest SDK.** The reference plugin is deliberately only an example, not a
  reusable authoring SDK.
- **Post-drain guest finalization.** Add a teardown export only after the
  processor lifecycle has an explicit point after the final `process` call and
  before final telemetry collection.

[parent]: https://github.com/open-telemetry/otel-arrow/issues/2973
[wit]: https://github.com/open-telemetry/otel-arrow/issues/3227
