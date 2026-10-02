// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Host state and native kernel implementations.
//!
//! The host owns OTAP records behind a host-managed pdata resource in
//! wasmtime's [`ResourceTable`]. Guests receive only that resource handle and
//! orchestrate kernels that execute natively here.

use arrow::array::{Array, AsArray, BooleanArray, DictionaryArray, StringArray};
use arrow::datatypes::{
    ArrowDictionaryKeyType, ArrowNativeType, DataType, Int8Type, Int16Type, Int32Type, Int64Type,
    UInt8Type, UInt16Type, UInt32Type, UInt64Type,
};
use wasmtime::component::{Accessor, HasSelf, Linker, Resource, ResourceTable};
use wasmtime_wasi::cli::{WasiCliCtx, WasiCliCtxView, WasiCliView};
use wasmtime_wasi::p3::bindings::clocks::{monotonic_clock, system_clock, types as clock_types};

use crate::bindings::otel::otap_dataflow_plugin::host_services::{self, LogLevel};
use crate::bindings::otel::otap_dataflow_plugin::otel_kernels::{self, AttrScope};
use crate::processor::WASM_PROCESSOR_URN;
use otel_arrow_dfe_pdata::OtapArrowRecords;
use otel_arrow_dfe_pdata::otap::filter::{IdBitmapPool, filter_otap_batch};
use otel_arrow_dfe_telemetry::Level;

// Emit guest-forwarded telemetry under the same component scope (URN +
// target) that `processor.rs` uses for native `wasm_processor` events, so a
// guest log is routed and attributed exactly like a native processor log
// rather than being tagged with this crate's package name.
otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = WASM_PROCESSOR_URN,
    target = "otel.processor.wasm_processor",
);

/// Node name reported for host state that was not constructed for a specific
/// pipeline node (unit tests, and the `Default`/`new` convenience paths).
const UNATTRIBUTED_NODE: &str = "unattributed";

/// Error returned when a guest attempts a future clock wait.
pub(crate) const WASI_CLOCK_WAIT_UNSUPPORTED: &str = "WASI clock waits are unavailable to plugins";

/// Maximum number of distinct guest-created counter names accepted per
/// plugin instance. See the `counter-add` doc comment in `wit/plugin.wit`
/// for the enforced policy: calls for names beyond this bound are accepted
/// (they do not trap) but silently no-op.
const MAX_GUEST_COUNTERS: usize = 16;
/// Maximum length of a guest-created counter name retained by the host.
const MAX_GUEST_COUNTER_NAME_LEN: usize = 256;
/// Maximum length of a guest `log` message retained by the host. Longer
/// messages are truncated (not dropped, so operators keep partial context)
/// before being forwarded to telemetry. See the `log` doc comment in
/// `wit/plugin.wit`.
///
/// Transient argument copies are bounded separately by
/// [`MAX_GUEST_HOSTCALL_BYTES`] and [`GuestCopyBudget`].
const MAX_GUEST_LOG_MESSAGE_LEN: usize = 4096;
/// Marker appended to guest log messages truncated by the host.
const GUEST_LOG_TRUNCATION_MARKER: &str = " ...[truncated]";
/// Canonical-ABI input-byte budget per lift, checked before string allocation.
/// This world exchanges control strings, not bulk telemetry payloads.
/// Non-UTF-8 canonical encodings can expand during conversion to UTF-8.
pub(crate) const MAX_GUEST_HOSTCALL_BYTES: usize = 16 * 1024;
/// Aggregate string-copy allowance per guest entry, also used as the
/// instance-wide byte bucket's burst capacity.
const GUEST_COPY_BYTE_BURST: usize = 2 * 1024 * 1024;
/// Sustained string-copy allowance across guest entries.
const GUEST_COPY_BYTES_PER_SEC: f64 = 1024.0 * 1024.0;
/// Maximum linear memory, in bytes, a single guest instance may grow to.
///
/// A guest that exceeds this sees `memory.grow` fail, and the guest allocator
/// should turn that failure into a clean trap.
///
/// TODO: Like `GUEST_FUEL_PER_CALL` in `processor.rs`, this is a placeholder
/// rather than a profiled budget, and is not yet configurable per
/// plugin/pipeline. Revisit as part of the limits-and-cache follow-on work.
const MAX_GUEST_MEMORY_BYTES: usize = 64 * 1024 * 1024;
/// Maximum number of core-Wasm linear memories a guest component may create.
///
/// Keeping this at one makes [`MAX_GUEST_MEMORY_BYTES`] an aggregate
/// per-instance memory cap rather than a per-memory cap that can be multiplied
/// by a component containing many core modules.
const MAX_GUEST_MEMORIES: usize = 1;
/// Maximum number of entries retained in the component resource table.
///
/// This bounds host-managed `pdata` handles, not core-Wasm tables (which are
/// bounded separately by [`wasmtime::ResourceLimiter`]). Retaining a handle
/// retains its complete host-side pdata batch.
const MAX_GUEST_TABLE_ELEMENTS: usize = 10_000;
/// Maximum number of core-Wasm tables a guest component may create.
///
/// Components produced by the reference toolchain use two tables, so this
/// allows that valid layout while keeping the aggregate table budget bounded.
const MAX_GUEST_TABLES: usize = 2;
/// Maximum number of elements in each core-Wasm table.
///
/// Combined with [`MAX_GUEST_TABLES`], this preserves the 10,000-element
/// aggregate table budget while preventing additional tables from multiplying
/// the previous per-table limit.
const MAX_GUEST_CORE_TABLE_ELEMENTS: usize = MAX_GUEST_TABLE_ELEMENTS / MAX_GUEST_TABLES;
/// Maximum number of native host-kernel calls allowed during one guest entry
/// point. This is a provisional invocation-count guard, not a CPU-time bound.
///
/// TODO: Revisit in the upcoming resource-limits PR. Kernel costs depend on
/// input size, and fuel/epochs cannot interrupt a synchronous host kernel.
/// Define limits using representative plugin workloads, data-size/work
/// accounting, and cooperative execution as needed. This cap does not
/// guarantee pipeline responsiveness.
const MAX_KERNEL_CALLS_PER_GUEST_CALL: u64 = 100_000;
/// Version of the host-services ABI this host implements, returned by
/// `host-services.host-abi-version`. Guests can branch on (or reject) it from
/// `initialize`. Bump whenever the observable behavior of an existing
/// `host-services` operation changes; adding a new operation is already a
/// world-level change that old guests link against unchanged.
const HOST_SERVICES_ABI_VERSION: u32 = 2;
/// Sustained call rate (calls per second) allowed through the shared
/// `log`/`counter-add`/`get-config` token bucket, once the burst allowance is
/// exhausted. See [`HostServiceRateLimiter`].
///
/// TODO: This value (like `GUEST_FUEL_PER_CALL` in `processor.rs`) is a
/// placeholder, not derived from profiling or a documented budget. Revisit
/// alongside that constant as part of the limits-and-cache follow-on work.
const HOST_SERVICE_CALL_RATE_PER_SEC: f64 = 1_000.0;
/// Burst allowance (in call "tokens") for the shared token bucket: the number
/// of calls a guest may make back-to-back before being throttled down to the
/// sustained rate above. See [`HostServiceRateLimiter`].
const HOST_SERVICE_CALL_BURST: f64 = 2_000.0;

/// Continuously-refilling token-bucket rate limiter shared by every
/// `host-services` operation that copies guest-controlled data or produces
/// host telemetry (`get-config`, `log`, `counter-add`).
///
/// otel-arrow pipeline nodes (and therefore `WasmProcessor`/`HostState`
/// instances) are long-lived telemetry-agent processes, not short batch
/// jobs, so a budget that resets to a fixed size on every `process` call
/// would either be too generous (if sized to tolerate one busy call) or
/// too strict (if sized to bound sustained throughput, it would also cap
/// a single legitimately bursty call). A token bucket instead bounds the
/// guest's *sustained* host-service call rate to
/// `HOST_SERVICE_CALL_RATE_PER_SEC` while still tolerating short bursts up
/// to `HOST_SERVICE_CALL_BURST`, independent of how calls are distributed
/// across `initialize`/`process` invocations over the instance's lifetime.
///
/// This bounds accepted calls and telemetry volume, not argument copies:
/// even rejected calls have already lifted their arguments. Those copies
/// are charged separately to [`GuestCopyBudget`].
struct HostServiceRateLimiter {
    /// Currently available call tokens, in `[0.0, HOST_SERVICE_CALL_BURST]`.
    tokens: f64,
    /// Monotonic instant tokens were last refilled up to. [`std::time::Instant`]
    /// is monotonic and non-decreasing, so a system clock change cannot hand
    /// the guest extra tokens or stall refills.
    last_refill: std::time::Instant,
}

/// Bounds string-copy work, including calls rejected by the telemetry limiter.
///
/// Import arguments have already been lifted when charged, so exhaustion
/// traps rather than no-opping and allowing further copies. The final lift
/// has at most `MAX_GUEST_HOSTCALL_BYTES` encoded input bytes. The per-entry
/// budget never refills, even when slow host calls replenish the lifetime bucket.
struct GuestCopyBudget {
    remaining: usize,
    tokens: f64,
    last_refill: std::time::Instant,
}

impl GuestCopyBudget {
    fn new() -> Self {
        Self {
            remaining: GUEST_COPY_BYTE_BURST,
            tokens: GUEST_COPY_BYTE_BURST as f64,
            last_refill: std::time::Instant::now(),
        }
    }

    fn consume(&mut self, bytes: usize, now: std::time::Instant) -> wasmtime::Result<()> {
        let elapsed = now
            .saturating_duration_since(self.last_refill)
            .as_secs_f64();
        self.tokens =
            (self.tokens + elapsed * GUEST_COPY_BYTES_PER_SEC).min(GUEST_COPY_BYTE_BURST as f64);
        self.last_refill = now;
        if bytes > self.remaining || bytes as f64 > self.tokens {
            return Err(wasmtime::Error::msg(
                "guest string-copy byte budget exceeded",
            ));
        }
        self.remaining -= bytes;
        self.tokens -= bytes as f64;
        Ok(())
    }
}

impl HostServiceRateLimiter {
    /// Create a limiter with a full burst allowance available immediately
    /// (so a freshly instantiated plugin is not throttled before it has
    /// made any calls).
    fn new() -> Self {
        Self {
            tokens: HOST_SERVICE_CALL_BURST,
            last_refill: std::time::Instant::now(),
        }
    }

    /// Attempt to consume one call token as of `now`, refilling based on
    /// elapsed monotonic time since the last attempt (capped at the burst
    /// allowance). Returns `true` if a token was available and consumed
    /// (the call should proceed), `false` if the bucket is empty (the
    /// caller should no-op and count the rejection).
    ///
    /// Takes `now` as a parameter (rather than calling
    /// [`std::time::Instant::now`] internally) so tests can simulate the
    /// passage of time deterministically via `Instant + Duration` instead
    /// of real sleeps.
    fn try_consume(&mut self, now: std::time::Instant) -> bool {
        let elapsed = now
            .saturating_duration_since(self.last_refill)
            .as_secs_f64();
        self.tokens =
            (self.tokens + elapsed * HOST_SERVICE_CALL_RATE_PER_SEC).min(HOST_SERVICE_CALL_BURST);
        self.last_refill = now;
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// One drained snapshot of guest `counter-add` activity, folded into the
/// processor's telemetry counters after each guest call.
///
/// The two rejection reasons are kept apart because they call for different
/// operator responses: a name-length rejection means the plugin is emitting a
/// malformed name, while a cardinality rejection means the plugin is trying
/// to track more than `MAX_GUEST_COUNTERS` distinct counters.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct GuestCounterActivity {
    /// Calls accepted (name within both bounds).
    pub accepted: u64,
    /// Calls rejected because the name exceeded `MAX_GUEST_COUNTER_NAME_LEN`.
    pub rejected_name_len: u64,
    /// Calls rejected because accepting the name would exceed
    /// `MAX_GUEST_COUNTERS` distinct names.
    pub rejected_cardinality: u64,
    /// Sum of the values carried by the accepted calls.
    pub value: u64,
}

/// Host-owned data behind a host-managed `pdata` resource handle.
///
/// This is the concrete type mapped to the WIT `pdata` resource. Guests never
/// see its contents; they only pass the handle back to host kernels.
pub struct HostPdata {
    /// The OTAP payload that host kernels operate on.
    pub otap_batch: OtapArrowRecords,
}

/// Per-instance host state stored in the wasmtime [`wasmtime::Store`].
///
/// Holds the host-managed pdata resource table. This state is confined to a single
/// pipeline/core thread and is never shared across threads.
pub struct HostState {
    /// Resource table backing the `pdata` resource.
    pub table: ResourceTable,
    /// Inert WASI CLI state: no arguments, environment, cwd, or stdio.
    wasi_cli: WasiCliCtx,
    /// Origin used for guest-observable monotonic clock reads.
    ///
    /// Clock reads remain available for Rust `std`, while future waits trap in
    /// every guest entry point.
    ///
    /// TODO: Replace real-time reads with frozen host clocks when the plugin
    /// determinism policy is implemented.
    wasi_monotonic_origin: std::time::Instant,
    /// Reusable bitmap pool for OTAP child-batch filtering propagation.
    pub id_bitmap_pool: IdBitmapPool,
    /// Accumulator for total host kernel invocations during a single guest call.
    /// Drained and added to telemetry counters after each `run_guest` return.
    pub kernel_calls: u64,
    /// Raw configuration blob served verbatim by `host-services.get-config`.
    /// Fixed at construction time; never mutated afterwards.
    config: String,
    /// Pipeline node name this plugin instance belongs to. Attached to every
    /// guest-forwarded log and counter event so operators can tell *which*
    /// plugin instance emitted a record when several `wasm_processor` nodes
    /// run in the same process.
    node_name: String,
    /// Guest-created named counters, bounded to `MAX_GUEST_COUNTERS` distinct
    /// names (first-seen-wins). These are the authoritative per-name totals:
    /// they are snapshotted and emitted as structured telemetry on each
    /// `CollectTelemetry` tick (see [`HostState::report_guest_counters`]),
    /// deliberately *not* on every `counter-add` call, so a guest counting
    /// once per batch does not put a log record on the pdata hot path.
    guest_counters: Vec<(String, u64)>,
    /// Accumulator for total `counter-add` calls accepted (name within the
    /// bound) during a single guest call. Drained by the processor.
    pub counter_add_calls: u64,
    /// Accumulator for `counter-add` calls rejected because the counter name
    /// exceeded `MAX_GUEST_COUNTER_NAME_LEN`. Tracked separately from
    /// [`Self::counter_add_rejected_cardinality`] because the two need
    /// different operator responses (fix the plugin's name vs. reduce the
    /// plugin's counter count). Drained by the processor.
    pub counter_add_rejected_name_len: u64,
    /// Accumulator for `counter-add` calls rejected because they would
    /// introduce a distinct counter name beyond `MAX_GUEST_COUNTERS`.
    /// Drained by the processor.
    pub counter_add_rejected_cardinality: u64,
    /// Accumulator for values accepted through `counter-add` during a single
    /// guest call. Drained by the processor.
    pub counter_add_value: u64,
    /// Token-bucket rate limiter shared by `get-config`, `log`, and
    /// `counter-add`, scoped to this instance's whole lifetime (not reset
    /// per call). See [`HostServiceRateLimiter`].
    host_service_rate_limiter: HostServiceRateLimiter,
    guest_copy_budget: GuestCopyBudget,
    /// Accumulator for host-service calls rejected (silently no-op'd)
    /// because the shared rate limiter's token bucket was empty. Drained by
    /// the processor.
    pub host_service_calls_rejected: u64,
    /// Accumulator for `log` messages truncated to `MAX_GUEST_LOG_MESSAGE_LEN`
    /// before being forwarded to telemetry. Drained by the processor.
    pub log_message_truncated: u64,
    #[cfg(test)]
    test_limiter_now: Option<std::time::Instant>,
}

impl HostState {
    /// Create empty host state with an empty configuration blob.
    #[must_use]
    pub fn new() -> Self {
        Self::with_config(String::new())
    }

    /// Create host state that serves `config` from `host-services.get-config`.
    ///
    /// Guest telemetry from this instance is reported as `UNATTRIBUTED_NODE`;
    /// use [`HostState::with_config_for_node`] on real pipeline paths so guest
    /// records carry their node name.
    #[must_use]
    pub fn with_config(config: String) -> Self {
        Self::with_config_for_node(config, UNATTRIBUTED_NODE.to_string())
    }

    /// Create host state that serves `config` from `host-services.get-config`
    /// and attributes guest telemetry to the pipeline node named `node_name`.
    #[must_use]
    pub fn with_config_for_node(config: String, node_name: String) -> Self {
        let mut table = ResourceTable::new();
        table.set_max_capacity(MAX_GUEST_TABLE_ELEMENTS);
        Self {
            table,
            wasi_cli: WasiCliCtx::default(),
            wasi_monotonic_origin: std::time::Instant::now(),
            id_bitmap_pool: IdBitmapPool::new(),
            kernel_calls: 0,
            config,
            node_name,
            guest_counters: Vec::new(),
            counter_add_calls: 0,
            counter_add_rejected_name_len: 0,
            counter_add_rejected_cardinality: 0,
            counter_add_value: 0,
            host_service_rate_limiter: HostServiceRateLimiter::new(),
            guest_copy_budget: GuestCopyBudget::new(),
            host_service_calls_rejected: 0,
            log_message_truncated: 0,
            #[cfg(test)]
            test_limiter_now: None,
        }
    }

    /// Consume one call token from the shared `log`/`counter-add` rate
    /// limiter. Returns `true` if a token was available (and consumed, so
    /// the call should proceed), `false` if the bucket is currently empty
    /// (the caller should no-op and count the rejection).
    fn consume_host_service_call_budget(&mut self) -> bool {
        if self
            .host_service_rate_limiter
            .try_consume(self.limiter_now())
        {
            true
        } else {
            self.host_service_calls_rejected += 1;
            false
        }
    }

    fn limiter_now(&self) -> std::time::Instant {
        #[cfg(test)]
        let now = self
            .test_limiter_now
            .unwrap_or_else(std::time::Instant::now);
        #[cfg(not(test))]
        let now = std::time::Instant::now();
        now
    }

    fn consume_guest_copy_budget(&mut self, bytes: usize) -> wasmtime::Result<()> {
        self.guest_copy_budget.consume(bytes, self.limiter_now())
    }

    #[cfg(test)]
    fn set_test_limiter_now(&mut self, now: std::time::Instant) {
        self.test_limiter_now = Some(now);
    }

    /// Drain the per-call kernel call counter, returning the accumulated count
    /// and resetting to zero.
    pub fn drain_kernel_counters(&mut self) -> u64 {
        let calls = self.kernel_calls;
        self.kernel_calls = 0;
        calls
    }

    /// Reset per-entry work budgets without refilling lifetime rate limits.
    pub fn begin_guest_call(&mut self) {
        self.kernel_calls = 0;
        self.guest_copy_budget.remaining = GUEST_COPY_BYTE_BURST;
    }

    fn consume_kernel_call_budget(&mut self) -> wasmtime::Result<()> {
        if self.kernel_calls >= MAX_KERNEL_CALLS_PER_GUEST_CALL {
            return Err(wasmtime::Error::msg(format!(
                "guest kernel call limit exceeded: maximum {MAX_KERNEL_CALLS_PER_GUEST_CALL} calls per guest entry"
            )));
        }
        self.kernel_calls += 1;
        Ok(())
    }

    /// Drain the `counter-add` accumulators, returning
    /// `(accepted, rejected_name_len, rejected_cardinality, value)` and
    /// resetting all to zero.
    pub fn drain_counter_add_calls(&mut self) -> GuestCounterActivity {
        let drained = GuestCounterActivity {
            accepted: self.counter_add_calls,
            rejected_name_len: self.counter_add_rejected_name_len,
            rejected_cardinality: self.counter_add_rejected_cardinality,
            value: self.counter_add_value,
        };
        self.counter_add_calls = 0;
        self.counter_add_rejected_name_len = 0;
        self.counter_add_rejected_cardinality = 0;
        self.counter_add_value = 0;
        drained
    }

    /// Emit the current per-name guest counter totals as structured host
    /// telemetry, one record per name.
    ///
    /// Called on the processor's `CollectTelemetry` tick rather than from
    /// `counter-add` itself: a plugin that counts once per batch would
    /// otherwise put a log record on the pdata hot path, and the per-name
    /// totals are cumulative anyway, so emitting them at the telemetry
    /// collection cadence loses nothing. Cardinality of the `name` attribute
    /// is bounded by `MAX_GUEST_COUNTERS`.
    pub fn report_guest_counters(&self) {
        for (name, value) in &self.guest_counters {
            otel_info!(
                "wasm_processor.guest_counter",
                node = self.node_name.as_str(),
                name = name.as_str(),
                value = *value
            );
        }
    }

    /// Drain the host-service call-budget accumulators, returning
    /// `(calls_rejected, log_messages_truncated)` and resetting both to zero.
    pub fn drain_host_service_budget_calls(&mut self) -> (u64, u64) {
        let rejected = self.host_service_calls_rejected;
        let truncated = self.log_message_truncated;
        self.host_service_calls_rejected = 0;
        self.log_message_truncated = 0;
        (rejected, truncated)
    }

    /// Test/introspection helper: the raw blob served to the guest by
    /// `host-services.get-config`.
    #[cfg(test)]
    pub(crate) fn config_for_test(&self) -> &str {
        &self.config
    }

    /// Test/introspection helper: current value of guest counter `name`, or
    /// `None` if it was never created (including if it was rejected by the
    /// cardinality bound).
    #[cfg(test)]
    pub(crate) fn guest_counter(&self, name: &str) -> Option<u64> {
        self.guest_counters
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| *v)
    }

    /// Test/introspection helper: number of distinct guest counter names
    /// currently tracked.
    #[cfg(test)]
    pub(crate) fn guest_counter_count(&self) -> usize {
        self.guest_counters.len()
    }
}

impl Default for HostState {
    fn default() -> Self {
        Self::new()
    }
}

impl WasiCliView for HostState {
    fn cli(&mut self) -> WasiCliCtxView<'_> {
        WasiCliCtxView {
            ctx: &mut self.wasi_cli,
            table: &mut self.table,
        }
    }
}

impl clock_types::Host for HostState {}

impl system_clock::Host for HostState {
    fn now(&mut self) -> wasmtime::Result<system_clock::Instant> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .map_err(|error| {
                wasmtime::Error::msg(format!("system clock precedes Unix epoch: {error}"))
            })?;
        Ok(system_clock::Instant {
            seconds: now.as_secs().try_into()?,
            nanoseconds: now.subsec_nanos(),
        })
    }

    fn get_resolution(&mut self) -> wasmtime::Result<clock_types::Duration> {
        Ok(1)
    }
}

impl monotonic_clock::Host for HostState {
    fn now(&mut self) -> wasmtime::Result<monotonic_clock::Mark> {
        std::time::Instant::now()
            .saturating_duration_since(self.wasi_monotonic_origin)
            .as_nanos()
            .try_into()
            .map_err(wasmtime::Error::from)
    }

    fn get_resolution(&mut self) -> wasmtime::Result<clock_types::Duration> {
        Ok(1)
    }
}

impl monotonic_clock::HostWithStore<HostState> for HasSelf<HostState> {
    async fn wait_until(
        accessor: &Accessor<HostState, Self>,
        when: monotonic_clock::Mark,
    ) -> wasmtime::Result<()> {
        let elapsed = accessor.with(|mut access| {
            let state = access.get();
            std::time::Instant::now().saturating_duration_since(state.wasi_monotonic_origin)
        });
        let now: monotonic_clock::Mark = elapsed.as_nanos().try_into()?;
        if when <= now {
            return Ok(());
        }
        reject_wasi_clock_wait()
    }

    async fn wait_for(
        accessor: &Accessor<HostState, Self>,
        duration: clock_types::Duration,
    ) -> wasmtime::Result<()> {
        if duration == 0 {
            return Ok(());
        }
        let _ = accessor;
        reject_wasi_clock_wait()
    }
}

fn reject_wasi_clock_wait() -> wasmtime::Result<()> {
    Err(wasmtime::Error::msg(WASI_CLOCK_WAIT_UNSUPPORTED))
}

/// Link only the WASI clock interfaces with phase-aware wait behavior.
pub(crate) fn add_wasi_clocks_to_linker(linker: &mut Linker<HostState>) -> wasmtime::Result<()> {
    clock_types::add_to_linker::<HostState, HasSelf<HostState>>(linker, |state| state)?;
    monotonic_clock::add_to_linker::<HostState, HasSelf<HostState>>(linker, |state| state)?;
    system_clock::add_to_linker::<HostState, HasSelf<HostState>>(linker, |state| state)?;
    Ok(())
}

/// Cap guest linear memory and table growth.
///
/// Host-side string allocations have separate lifting and copy budgets.
impl wasmtime::ResourceLimiter for HostState {
    fn memory_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        // Returning `Ok(false)` makes `memory.grow` return -1 to the guest
        // (a normal allocation failure it can observe) rather than trapping
        // the store outright.
        Ok(desired <= MAX_GUEST_MEMORY_BYTES)
    }

    fn table_growing(
        &mut self,
        _current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        Ok(desired <= MAX_GUEST_CORE_TABLE_ELEMENTS)
    }

    fn memories(&self) -> usize {
        MAX_GUEST_MEMORIES
    }

    fn tables(&self) -> usize {
        MAX_GUEST_TABLES
    }
}

impl otel_kernels::HostPdata for HostState {
    fn drop(&mut self, data: Resource<HostPdata>) -> wasmtime::Result<()> {
        let _ = self.table.delete(data)?;
        Ok(())
    }
}

impl otel_kernels::Host for HostState {}
impl host_services::Host for HostState {}

impl HostState {
    fn pdata_num_rows(&mut self, data: Resource<HostPdata>) -> wasmtime::Result<u32> {
        self.consume_kernel_call_budget()?;
        // A stale/forged handle is a guest contract violation: trap the guest
        // rather than panicking the host process.
        let host_pdata = self
            .table
            .get(&data)
            .map_err(|e| wasmtime::Error::msg(format!("invalid pdata resource handle: {e}")))?;
        Ok(host_pdata
            .otap_batch
            .root_record_batch()
            .map_or(0, |batch| batch.num_rows() as u32))
    }

    fn filter_by_attribute_eq(
        &mut self,
        data: Resource<HostPdata>,
        scope: AttrScope,
        key: String,
        value: String,
    ) -> wasmtime::Result<Resource<HostPdata>> {
        self.consume_guest_copy_budget(key.len() + value.len())?;
        self.consume_kernel_call_budget()?;
        // Consume the input handle and take ownership of the batch. Invalid
        // handles are a contract violation and trap the guest instead of
        // silently dropping data (or panicking the host).
        let input = self
            .table
            .delete(data)
            .map_err(|e| wasmtime::Error::msg(format!("invalid pdata resource handle: {e}")))?
            .otap_batch;

        // Every failure below is reachable from guest-controlled arguments
        // (an unsupported scope, or a key absent from this batch's schema),
        // so each returns a trap: the guest violated the kernel contract, and
        // trapping fails only that plugin call, whereas a panic would take
        // the whole collector process down -- and, during `Drop`-driven
        // teardown, could escalate to a double-panic abort.
        let result = match scope {
            AttrScope::Resource | AttrScope::Scope => {
                return Err(wasmtime::Error::msg(format!(
                    "unsupported attr scope {scope:?}: this experimental slice currently supports only record scope"
                )));
            }
            AttrScope::Record => filter_otap_batch_by_column_eq(
                &input,
                &key,
                &value,
                &mut self.id_bitmap_pool,
            )
            .map_err(|error| {
                wasmtime::Error::msg(format!(
                    "filter-by-attribute-eq failed for key {key:?} and value {value:?}: {error}"
                ))
            })?,
        };

        self.table
            .push(HostPdata { otap_batch: result })
            .map_err(|e| wasmtime::Error::msg(format!("pdata resource table push failed: {e}")))
    }

    fn host_abi_version(&mut self) -> wasmtime::Result<u32> {
        // Deliberately outside the token bucket and free of guest-controlled
        // input: it returns a constant, allocates nothing, and a guest needs
        // it before it can decide whether talking to this host is safe at all.
        Ok(HOST_SERVICES_ABI_VERSION)
    }

    fn get_config(&mut self) -> wasmtime::Result<String> {
        // Rate-limited like the other host services: each call clones the
        // whole config blob, so an unbounded call rate is an unbounded host
        // allocation rate. A throttled call returns an empty string rather
        // than trapping, consistent with `log`/`counter-add` no-op semantics;
        // guests are expected to call this once, from `initialize`.
        if !self.consume_host_service_call_budget() {
            return Ok(String::new());
        }
        self.consume_guest_copy_budget(self.config.len())?;
        Ok(self.config.clone())
    }

    fn log(&mut self, level: LogLevel, mut message: String) -> wasmtime::Result<()> {
        self.consume_guest_copy_budget(message.len())?;
        if !self.consume_host_service_call_budget() {
            return Ok(());
        }
        if truncate_guest_log_message(&mut message) {
            self.log_message_truncated += 1;
        }
        // Route every guest log line through this host's own component-scoped
        // telemetry macros -- never a raw `println!`/stdout write -- and tag
        // it with the emitting node so guest records are routed like native
        // processor logs while remaining attributable to a specific plugin
        // instance.
        let node = self.node_name.as_str();
        match level {
            LogLevel::Trace => otel_event!(
                Level::TRACE,
                "wasm_processor.guest_log",
                node = node,
                message = message
            ),
            LogLevel::Debug => {
                otel_debug!("wasm_processor.guest_log", node = node, message = message)
            }
            LogLevel::Info => {
                otel_info!("wasm_processor.guest_log", node = node, message = message)
            }
            LogLevel::Warn => {
                otel_warn!("wasm_processor.guest_log", node = node, message = message)
            }
            LogLevel::Error => {
                otel_error!("wasm_processor.guest_log", node = node, message = message)
            }
        }

        Ok(())
    }

    fn counter_add(&mut self, name: String, value: u64) -> wasmtime::Result<()> {
        self.consume_guest_copy_budget(name.len())?;
        if !self.consume_host_service_call_budget() {
            return Ok(());
        }

        if name.len() > MAX_GUEST_COUNTER_NAME_LEN {
            self.counter_add_rejected_name_len += 1;
            return Ok(());
        }

        if let Some((_, existing)) = self.guest_counters.iter_mut().find(|(n, _)| *n == name) {
            // Saturate rather than `+=`: `value` is fully guest-controlled, so
            // an unchecked add lets a guest panic the host (overflow checks on
            // in debug/test) or silently wrap the metric (release).
            *existing = existing.saturating_add(value);
            self.counter_add_calls += 1;
            self.counter_add_value = self.counter_add_value.saturating_add(value);
            return Ok(());
        }

        if self.guest_counters.len() >= MAX_GUEST_COUNTERS {
            // Cardinality bound reached: silently no-op per the documented
            // policy in `wit/plugin.wit` rather than trapping the guest. The
            // rejection is still visible to operators through the
            // `guest_counter_add_rejected_cardinality` metric.
            self.counter_add_rejected_cardinality += 1;
            return Ok(());
        }

        self.guest_counters.push((name, value));
        self.counter_add_calls += 1;
        self.counter_add_value = self.counter_add_value.saturating_add(value);
        Ok(())
    }
}

/// Truncate an oversized guest log message, including the marker in the limit.
fn truncate_guest_log_message(message: &mut String) -> bool {
    if message.len() <= MAX_GUEST_LOG_MESSAGE_LEN {
        return false;
    }

    let mut truncate_at = MAX_GUEST_LOG_MESSAGE_LEN - GUEST_LOG_TRUNCATION_MARKER.len();
    while truncate_at > 0 && !message.is_char_boundary(truncate_at) {
        truncate_at -= 1;
    }
    message.truncate(truncate_at);
    message.push_str(GUEST_LOG_TRUNCATION_MARKER);
    true
}

impl otel_kernels::HostWithStore<HostState> for HasSelf<HostState> {
    async fn pdata_num_rows(
        accessor: &Accessor<HostState, Self>,
        data: Resource<HostPdata>,
    ) -> wasmtime::Result<u32> {
        accessor.with(|mut access| access.get().pdata_num_rows(data))
    }

    async fn filter_by_attribute_eq(
        accessor: &Accessor<HostState, Self>,
        data: Resource<HostPdata>,
        scope: AttrScope,
        key: String,
        value: String,
    ) -> wasmtime::Result<Resource<HostPdata>> {
        accessor.with(|mut access| access.get().filter_by_attribute_eq(data, scope, key, value))
    }
}

impl host_services::HostWithStore<HostState> for HasSelf<HostState> {
    async fn host_abi_version(accessor: &Accessor<HostState, Self>) -> wasmtime::Result<u32> {
        accessor.with(|mut access| access.get().host_abi_version())
    }

    async fn get_config(accessor: &Accessor<HostState, Self>) -> wasmtime::Result<String> {
        accessor.with(|mut access| access.get().get_config())
    }

    async fn log(
        accessor: &Accessor<HostState, Self>,
        level: LogLevel,
        message: String,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut access| access.get().log(level, message))
    }

    async fn counter_add(
        accessor: &Accessor<HostState, Self>,
        name: String,
        value: u64,
    ) -> wasmtime::Result<()> {
        accessor.with(|mut access| access.get().counter_add(name, value))
    }
}

/// Native OTel-semantic filter kernel: keep rows whose `key` column equals
/// `value` (string comparison).
///
/// Handles plain `Utf8`, `LargeUtf8`, and dictionary-encoded string columns by
/// using a dictionary-aware comparison fast path and falling back to `Utf8`
/// casting for other encodings.
///
fn filter_otap_batch_by_column_eq(
    otap_batch: &OtapArrowRecords,
    key: &str,
    value: &str,
    id_bitmap_pool: &mut IdBitmapPool,
) -> Result<OtapArrowRecords, String> {
    let Some(root_batch) = otap_batch.root_record_batch() else {
        return Err("root record batch not present for filtering".to_string());
    };

    let Some(column) = root_batch.column_by_name(key) else {
        return Err(format!(
            "attribute column {key:?} not present in root record batch"
        ));
    };

    let mask = if let Some(mask) = dictionary_string_eq_mask(column.as_ref(), value)? {
        mask
    } else {
        let utf8 = if column.data_type() == &DataType::Utf8 {
            column.clone()
        } else {
            match arrow_cast::cast(column, &DataType::Utf8) {
                Ok(arr) => arr,
                Err(error) => {
                    return Err(format!(
                        "failed to cast attribute column {key:?} to Utf8 for comparison: {error}"
                    ));
                }
            }
        };

        let scalar = StringArray::new_scalar(value);
        match arrow::compute::kernels::cmp::eq(&utf8, &scalar) {
            Ok(mask) => mask,
            Err(error) => {
                return Err(format!(
                    "failed to compare attribute column {key:?} against value {value:?}: {error}"
                ));
            }
        }
    };

    filter_otap_batch(&mask, otap_batch, id_bitmap_pool).map_err(|error| {
        format!("failed to filter OTAP payload for key {key:?} and value {value:?}: {error}")
    })
}

pub(crate) fn dictionary_string_eq_mask(
    column: &dyn Array,
    value: &str,
) -> Result<Option<BooleanArray>, String> {
    let DataType::Dictionary(key_type, value_type) = column.data_type() else {
        return Ok(None);
    };

    if !matches!(**value_type, DataType::Utf8 | DataType::LargeUtf8) {
        return Ok(None);
    }

    macro_rules! dispatch_key_type {
        ($key_ty:ty) => {{
            let dict = column
                .as_any()
                .downcast_ref::<DictionaryArray<$key_ty>>()
                .ok_or_else(|| "failed to downcast dictionary column".to_string())?;
            Ok(Some(dictionary_eq_mask_impl(dict, value)))
        }};
    }

    match key_type.as_ref() {
        DataType::Int8 => dispatch_key_type!(Int8Type),
        DataType::Int16 => dispatch_key_type!(Int16Type),
        DataType::Int32 => dispatch_key_type!(Int32Type),
        DataType::Int64 => dispatch_key_type!(Int64Type),
        DataType::UInt8 => dispatch_key_type!(UInt8Type),
        DataType::UInt16 => dispatch_key_type!(UInt16Type),
        DataType::UInt32 => dispatch_key_type!(UInt32Type),
        DataType::UInt64 => dispatch_key_type!(UInt64Type),
        _ => Ok(None),
    }
}

fn dictionary_eq_mask_impl<K: ArrowDictionaryKeyType>(
    dict: &DictionaryArray<K>,
    value: &str,
) -> BooleanArray
where
    K::Native: ArrowNativeType,
{
    let keys = dict.keys();
    let mut matches = Vec::with_capacity(dict.len());
    match dict.values().data_type() {
        DataType::Utf8 => {
            let values = dict.values().as_string::<i32>();
            for i in 0..dict.len() {
                if keys.is_null(i) {
                    matches.push(false);
                    continue;
                }
                let key_index = keys.value(i).as_usize();
                matches.push(!values.is_null(key_index) && values.value(key_index) == value);
            }
        }
        DataType::LargeUtf8 => {
            let values = dict.values().as_string::<i64>();
            for i in 0..dict.len() {
                if keys.is_null(i) {
                    matches.push(false);
                    continue;
                }
                let key_index = keys.value(i).as_usize();
                matches.push(!values.is_null(key_index) && values.value(key_index) == value);
            }
        }
        _ => {
            matches.resize(dict.len(), false);
        }
    }
    BooleanArray::from(matches)
}

#[cfg(test)]
mod tests {
    use super::*;
    use arrow::array::{Array, DictionaryArray, RecordBatch, StringArray, UInt16Array};
    use arrow::datatypes::{Field, Schema, UInt8Type};
    use otel_arrow_dfe_pdata::otap::Logs;
    use otel_arrow_dfe_pdata::proto::OtlpProtoMessage;
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
    use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::LogRecord;
    use otel_arrow_dfe_pdata::testing::round_trip::{otap_to_otlp, to_otap_logs};
    use std::sync::Arc;

    // Thin wrappers over the host-service imports. Every one of them now
    // returns `wasmtime::Result` (host imports are generated as trappable so
    // guest-driven failures become traps rather than host panics), but none
    // of these three can actually fail: they exist to keep the assertions
    // below focused on the bound/rate-limit behavior under test.
    fn get_config(host: &mut HostState) -> String {
        host.get_config().expect("get-config is infallible")
    }

    fn log(host: &mut HostState, level: LogLevel, message: String) {
        host.log(level, message).expect("log is infallible");
    }

    fn counter_add(host: &mut HostState, name: &str, value: u64) {
        host.counter_add(name.to_string(), value)
            .expect("counter-add is infallible");
    }

    fn batch_with_severity(values: &[&str]) -> OtapArrowRecords {
        let schema = Schema::new(vec![
            Field::new("id", DataType::UInt16, true),
            Field::new("severity_text", DataType::Utf8, true),
        ]);
        let record_batch = RecordBatch::try_new(
            Arc::new(schema),
            vec![
                Arc::new(UInt16Array::from(
                    (0..values.len() as u16).collect::<Vec<_>>(),
                )),
                Arc::new(StringArray::from(values.to_vec())),
            ],
        )
        .unwrap();
        let mut otap = OtapArrowRecords::Logs(Logs::default());
        otap.set(
            otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType::Logs,
            record_batch,
        )
        .expect("set logs root batch");
        otap
    }

    fn severity_values(otap_batch: &OtapArrowRecords) -> Vec<String> {
        let col = otap_batch
            .root_record_batch()
            .expect("root batch present")
            .column_by_name("severity_text")
            .unwrap();
        let arr = arrow_cast::cast(col, &DataType::Utf8).unwrap();
        let strings = arr.as_any().downcast_ref::<StringArray>().unwrap();
        (0..strings.len())
            .map(|i| strings.value(i).to_string())
            .collect()
    }

    /// Scenario: Record-scope filtering receives matching and non-matching
    /// severity values.
    /// Guarantees: Only rows matching `severity_text == "ERROR"` are retained.
    #[test]
    fn filters_matching_rows() {
        let batch = batch_with_severity(&["ERROR", "INFO", "ERROR", "WARN"]);
        let mut pool = IdBitmapPool::new();
        let out = filter_otap_batch_by_column_eq(&batch, "severity_text", "ERROR", &mut pool)
            .expect("filter should succeed");
        assert_eq!(out.root_record_batch().expect("root batch").num_rows(), 2);
        assert_eq!(severity_values(&out), vec!["ERROR", "ERROR"]);
    }

    /// Scenario: Record-scope filtering references an attribute key that does
    /// not exist in the root record batch.
    /// Guarantees: The kernel reports an explicit error instead of silently
    /// passing data through unchanged.
    #[test]
    fn missing_column_is_error() {
        let batch = batch_with_severity(&["ERROR", "INFO"]);
        let mut pool = IdBitmapPool::new();
        let result = filter_otap_batch_by_column_eq(&batch, "does_not_exist", "ERROR", &mut pool);
        assert!(
            result.is_err(),
            "missing attribute key should be reported explicitly"
        );
    }

    /// Scenario: Record-scope filtering is invoked on a dictionary-encoded
    /// `severity_text` column.
    /// Guarantees: The kernel can cast dictionary-encoded values and keep only
    /// matching rows.
    #[test]
    fn handles_dictionary_encoded_columns() {
        // OTAP severity_text is typically dictionary-encoded; the kernel must
        // still compare correctly after casting to Utf8.
        let dict: DictionaryArray<UInt8Type> = vec!["ERROR", "INFO", "ERROR"].into_iter().collect();
        let schema = Schema::new(vec![Field::new(
            "severity_text",
            dict.data_type().clone(),
            true,
        )]);
        let batch = RecordBatch::try_new(Arc::new(schema), vec![Arc::new(dict)]).unwrap();
        let mut otap = OtapArrowRecords::Logs(Logs::default());
        otap.set(
            otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType::Logs,
            batch,
        )
        .expect("set logs root batch");
        let mut pool = IdBitmapPool::new();
        let out = filter_otap_batch_by_column_eq(&otap, "severity_text", "ERROR", &mut pool)
            .expect("filter should succeed");
        assert_eq!(out.root_record_batch().expect("root batch").num_rows(), 2);
    }

    /// Scenario: Record-scope filtering receives a dictionary-encoded string
    /// column with null keys and null dictionary values.
    /// Guarantees: Null keys and null dictionary values do not match the target
    /// value and are excluded from filtered results.
    #[test]
    fn dictionary_encoded_nulls_do_not_match() {
        let dict: DictionaryArray<UInt8Type> =
            vec![Some("ERROR"), None, Some("INFO"), Some("ERROR"), None]
                .into_iter()
                .collect();
        let schema = Schema::new(vec![Field::new(
            "severity_text",
            dict.data_type().clone(),
            true,
        )]);
        let batch = RecordBatch::try_new(Arc::new(schema), vec![Arc::new(dict)]).unwrap();
        let mut otap = OtapArrowRecords::Logs(Logs::default());
        otap.set(
            otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType::Logs,
            batch,
        )
        .expect("set logs root batch");
        let mut pool = IdBitmapPool::new();
        let out = filter_otap_batch_by_column_eq(&otap, "severity_text", "ERROR", &mut pool)
            .expect("filter should succeed");
        assert_eq!(out.root_record_batch().expect("root batch").num_rows(), 2);
        assert_eq!(severity_values(&out), vec!["ERROR", "ERROR"]);
    }

    /// Scenario: Root log records are filtered from a payload that includes
    /// per-record log attributes.
    /// Guarantees: Child log attribute rows are filtered with the same parent
    /// selection, so only attributes of surviving records remain.
    #[test]
    fn filtering_preserves_log_attribute_relationships() {
        let input = to_otap_logs(vec![
            LogRecord::build()
                .severity_text("ERROR")
                .attributes(vec![KeyValue::new("k", AnyValue::new_string("e0"))])
                .finish(),
            LogRecord::build()
                .severity_text("INFO")
                .attributes(vec![KeyValue::new("k", AnyValue::new_string("i1"))])
                .finish(),
            LogRecord::build()
                .severity_text("ERROR")
                .attributes(vec![KeyValue::new("k", AnyValue::new_string("e2"))])
                .finish(),
        ]);

        let mut pool = IdBitmapPool::new();
        let output =
            filter_otap_batch_by_column_eq(&input, "severity_text", "ERROR", &mut pool).unwrap();
        let OtlpProtoMessage::Logs(logs) = otap_to_otlp(&output) else {
            panic!("expected logs payload");
        };

        let records = &logs.resource_logs[0].scope_logs[0].log_records;
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].severity_text, "ERROR");
        assert_eq!(records[1].severity_text, "ERROR");
        let attr_values: Vec<String> =
            records
                .iter()
                .map(|record| {
                    record.attributes[0]
                        .value
                        .as_ref()
                        .expect("attribute value")
                })
                .map(|value| {
                    match value.value.as_ref().expect("typed attribute value") {
                otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::any_value::Value::StringValue(
                    s,
                ) => s.clone(),
                other => panic!("expected string attribute value, got {other:?}"),
            }
                })
                .collect();
        assert_eq!(attr_values, vec!["e0", "e2"]);
    }

    /// Scenario: Guest calls `pdata-num-rows` or `filter-by-attribute-eq`
    /// with record scope.
    /// Guarantees: Each kernel invocation increments `kernel_calls` by one.
    #[test]
    fn kernel_calls_incremented_per_invocation() {
        let mut host = HostState::new();
        let batch = batch_with_severity(&["ERROR", "INFO", "WARN"]);

        // pdata_num_rows counts as one kernel call.
        let h0 = host
            .table
            .push(HostPdata {
                otap_batch: batch.clone(),
            })
            .expect("push batch");
        let _ = host.pdata_num_rows(h0);
        assert_eq!(host.kernel_calls, 1);

        // filter_by_attribute_eq (record scope) also counts as one kernel call.
        let h1 = host
            .table
            .push(HostPdata { otap_batch: batch })
            .expect("push batch");
        let out_handle = host
            .filter_by_attribute_eq(
                h1,
                AttrScope::Record,
                "severity_text".to_string(),
                "ERROR".to_string(),
            )
            .expect("record-scope filter succeeds");
        let _ = host.table.delete(out_handle).expect("delete output handle");
        assert_eq!(host.kernel_calls, 2);
    }

    /// Scenario: `drain_kernel_counters` is called after accumulating counts.
    /// Guarantees: The returned value equals the accumulated count and the
    /// accumulator is reset to zero for the next call.
    #[test]
    fn drain_kernel_counters_resets_to_zero() {
        let mut host = HostState::new();
        host.kernel_calls = 5;

        let calls = host.drain_kernel_counters();
        assert_eq!(calls, 5);
        assert_eq!(host.kernel_calls, 0, "kernel_calls must reset after drain");
    }

    /// Scenario: A guest has already reached the native kernel-call budget for
    /// one entry point and requests another kernel operation.
    /// Guarantees: The extra operation traps before doing native work, while
    /// the accumulated calls remain available for telemetry.
    #[test]
    fn kernel_call_budget_traps_before_extra_work() {
        let mut host = HostState::new();
        host.kernel_calls = MAX_KERNEL_CALLS_PER_GUEST_CALL;
        let handle = host
            .table
            .push(HostPdata {
                otap_batch: batch_with_severity(&["ERROR"]),
            })
            .expect("push batch");

        let error = host
            .pdata_num_rows(handle)
            .expect_err("the kernel-call budget must trap");
        assert!(
            error
                .to_string()
                .contains("guest kernel call limit exceeded"),
            "trap should identify the exhausted kernel budget, got: {error}"
        );
        assert_eq!(host.kernel_calls, MAX_KERNEL_CALLS_PER_GUEST_CALL);
    }

    /// Scenario: A component resource table is filled to its configured
    /// capacity and then receives one additional resource.
    /// Guarantees: Retained pdata handles cannot grow the host table beyond
    /// `MAX_GUEST_TABLE_ELEMENTS`.
    #[test]
    fn resource_table_capacity_is_bounded() {
        let mut host = HostState::new();
        for i in 0..MAX_GUEST_TABLE_ELEMENTS {
            let _ = host
                .table
                .push(i)
                .expect("capacity should accept valid entries");
        }
        assert!(
            host.table.push(0u32).is_err(),
            "resource table must reject entries beyond its configured capacity"
        );
    }

    /// Scenario: A guest component requests core-Wasm memories and tables in
    /// the store.
    /// Guarantees: The resource limiter allows one memory and at most two
    /// tables, with the table element cap divided across those tables so
    /// resource limits cannot be multiplied by additional core modules.
    #[test]
    fn core_wasm_resource_counts_are_bounded() {
        let host = HostState::new();
        let limiter: &dyn wasmtime::ResourceLimiter = &host;
        assert_eq!(limiter.memories(), MAX_GUEST_MEMORIES);
        assert_eq!(limiter.tables(), MAX_GUEST_TABLES);
        assert!(MAX_GUEST_CORE_TABLE_ELEMENTS * limiter.tables() <= MAX_GUEST_TABLE_ELEMENTS);
    }

    /// Scenario: Guest requests `resource` or `scope` filtering in the current
    /// experimental vertical slice.
    /// Guarantees: Unsupported scopes return a trap to the guest instead of
    /// silently passing data through -- and, critically, without panicking
    /// the host process: this argument is entirely guest-controlled, so a
    /// panic here would let any plugin abort the collector.
    #[test]
    fn resource_and_scope_filter_traps() {
        let mut host = HostState::new();
        let handle = host
            .table
            .push(HostPdata {
                otap_batch: batch_with_severity(&["ERROR", "INFO", "WARN"]),
            })
            .expect("push input batch");

        let result = host.filter_by_attribute_eq(
            handle,
            AttrScope::Resource,
            "severity_text".to_string(),
            "ERROR".to_string(),
        );
        let error = result.expect_err("unsupported scope must trap the guest");
        assert!(
            error.to_string().contains("unsupported attr scope"),
            "trap should name the unsupported scope, got: {error}"
        );
    }

    /// Scenario: Guest asks `filter-by-attribute-eq` for an attribute key
    /// that is not present in the batch's schema.
    /// Guarantees: The host returns a trap rather than panicking. The key is
    /// guest-supplied and its presence depends on runtime data, so this is
    /// reachable in normal operation by any plugin, not just a malicious one.
    #[test]
    fn filter_on_absent_key_traps_without_panicking() {
        let mut host = HostState::new();
        let handle = host
            .table
            .push(HostPdata {
                otap_batch: batch_with_severity(&["ERROR"]),
            })
            .expect("push input batch");

        let result = host.filter_by_attribute_eq(
            handle,
            AttrScope::Record,
            "no_such_column".to_string(),
            "ERROR".to_string(),
        );
        let error = result.expect_err("absent key must trap the guest");
        assert!(
            error.to_string().contains("no_such_column"),
            "trap should name the offending key, got: {error}"
        );
    }

    /// Scenario: Guest passes an invalid pdata resource handle to
    /// `pdata-num-rows`.
    /// Guarantees: Invalid resource handles trap instead of being interpreted
    /// as empty data.
    #[test]
    fn invalid_handle_for_pdata_num_rows_traps() {
        let mut host = HostState::new();
        let invalid = Resource::<HostPdata>::new_own(u32::MAX);
        let result = host.pdata_num_rows(invalid);
        let error = result.expect_err("an invalid handle must trap the guest");
        assert!(
            error.to_string().contains("invalid pdata resource handle"),
            "trap should identify the handle problem, got: {error}"
        );
    }

    /// Scenario: Guest passes an invalid pdata resource handle to
    /// `filter-by-attribute-eq`.
    /// Guarantees: Invalid resource handles trap instead of returning fabricated
    /// filtered output.
    #[test]
    fn invalid_handle_for_filter_traps() {
        let mut host = HostState::new();
        let invalid = Resource::<HostPdata>::new_own(u32::MAX);
        let result = host.filter_by_attribute_eq(
            invalid,
            AttrScope::Record,
            "severity_text".to_string(),
            "ERROR".to_string(),
        );
        let error = result.expect_err("an invalid handle must trap the guest");
        assert!(
            error.to_string().contains("invalid pdata resource handle"),
            "trap should identify the handle problem, got: {error}"
        );
    }

    /// Scenario: `HostState::default()` is used to construct initial state.
    /// Guarantees: The default construction produces the same empty state as
    /// `HostState::new()` -- zero kernel counters and an empty resource table.
    #[test]
    fn host_state_default_matches_new() {
        let from_default = HostState::default();
        assert_eq!(from_default.kernel_calls, 0);
    }

    /// Scenario: `host-services.get-config` is called against a `HostState`
    /// constructed with a known configuration blob.
    /// Guarantees: The exact string passed to `with_config` is returned
    /// verbatim, unmodified, on every call.
    #[test]
    fn get_config_returns_configured_blob() {
        let mut host = HostState::with_config("{\"level\":\"ERROR\"}".to_string());
        assert_eq!(
            get_config(&mut host),
            "{\"level\":\"ERROR\"}",
            "the blob is returned verbatim"
        );
        // Calling again returns the same value: read-only, fixed at
        // construction.
        assert_eq!(get_config(&mut host), "{\"level\":\"ERROR\"}");
    }

    /// Scenario: A guest calls `host-services.get-config` in a tight loop,
    /// past the shared token bucket's burst allowance.
    /// Guarantees: `get-config` draws on the same limiter as `log` and
    /// `counter-add`, so it cannot be used to drive an unbounded rate of
    /// host-side config clones; throttled calls return an empty string
    /// rather than trapping, and are counted as rejections.
    #[test]
    fn get_config_is_rate_limited() {
        let mut host = HostState::with_config("{\"level\":\"ERROR\"}".to_string());
        host.set_test_limiter_now(std::time::Instant::now());
        for _ in 0..HOST_SERVICE_CALL_BURST as usize {
            assert_eq!(get_config(&mut host), "{\"level\":\"ERROR\"}");
        }
        assert_eq!(
            get_config(&mut host),
            "",
            "a throttled get-config returns empty rather than trapping"
        );
        let (rejected, _truncated) = host.drain_host_service_budget_calls();
        assert_eq!(rejected, 1, "the throttled call is counted for operators");
    }

    /// Scenario: A guest asks the host which `host-services` ABI version it
    /// implements.
    /// Guarantees: A stable, non-zero version is returned, and asking does
    /// not consume the shared rate-limiter budget -- a guest must be able to
    /// check compatibility before deciding whether to use anything else.
    #[test]
    fn host_abi_version_is_reported_without_consuming_budget() {
        let mut host = HostState::new();
        for _ in 0..(HOST_SERVICE_CALL_BURST as usize + 100) {
            assert_eq!(
                host.host_abi_version().expect("abi version"),
                HOST_SERVICES_ABI_VERSION
            );
        }
        let (rejected, _truncated) = host.drain_host_service_budget_calls();
        assert_eq!(
            rejected, 0,
            "host-abi-version must not draw on the shared token bucket"
        );
    }

    /// Scenario: A guest calls `counter-add` twice for the same counter name.
    /// Guarantees: Values accumulate under that name and `counter_add_calls`
    /// tracks both accepted calls.
    #[test]
    fn counter_add_accumulates_same_name() {
        let mut host = HostState::new();
        counter_add(&mut host, "guest.calls", 3);
        counter_add(&mut host, "guest.calls", 4);
        assert_eq!(host.guest_counter("guest.calls"), Some(7));
        assert_eq!(host.guest_counter_count(), 1);
        let drained = host.drain_counter_add_calls();
        assert_eq!(
            drained.accepted, 2,
            "both calls for the same name are accepted"
        );
        assert_eq!(drained.rejected_name_len, 0);
        assert_eq!(drained.rejected_cardinality, 0);
        assert_eq!(drained.value, 7);
    }

    /// Scenario: A malicious or buggy guest calls `counter-add` with values
    /// that sum past `u64::MAX`, for both an existing and a fresh counter
    /// name.
    /// Guarantees: The host saturates instead of overflowing, so a guest
    /// cannot panic (debug builds) or silently corrupt telemetry (release
    /// builds) through unbounded counter values.
    #[test]
    fn counter_add_saturates_on_overflow() {
        let mut host = HostState::new();
        counter_add(&mut host, "guest.calls", u64::MAX);
        counter_add(&mut host, "guest.calls", 1);
        assert_eq!(
            host.guest_counter("guest.calls"),
            Some(u64::MAX),
            "per-name total saturates rather than wrapping"
        );

        counter_add(&mut host, "guest.other", u64::MAX);
        let drained = host.drain_counter_add_calls();
        assert_eq!(drained.accepted, 3);
        assert_eq!(drained.rejected_name_len, 0);
        assert_eq!(drained.rejected_cardinality, 0);
        assert_eq!(
            drained.value,
            u64::MAX,
            "aggregate value saturates rather than wrapping"
        );
    }

    /// Scenario: A guest calls `counter-add` with more distinct names than
    /// `MAX_GUEST_COUNTERS` allows.
    /// Guarantees: Only the first `MAX_GUEST_COUNTERS` distinct names are
    /// tracked; excess distinct names are silently rejected (not trapped) and
    /// do not create unbounded counter cardinality.
    #[test]
    fn counter_add_enforces_cardinality_bound() {
        let mut host = HostState::new();
        for i in 0..(MAX_GUEST_COUNTERS + 5) {
            counter_add(&mut host, &format!("counter.{i}"), 1);
        }
        assert_eq!(
            host.guest_counter_count(),
            MAX_GUEST_COUNTERS,
            "distinct counter names must not exceed the documented bound"
        );
        // The first MAX_GUEST_COUNTERS names were accepted.
        for i in 0..MAX_GUEST_COUNTERS {
            assert_eq!(host.guest_counter(&format!("counter.{i}")), Some(1));
        }
        // Names beyond the bound were rejected: never created.
        for i in MAX_GUEST_COUNTERS..(MAX_GUEST_COUNTERS + 5) {
            assert_eq!(host.guest_counter(&format!("counter.{i}")), None);
        }
        let drained = host.drain_counter_add_calls();
        assert_eq!(drained.accepted as usize, MAX_GUEST_COUNTERS);
        assert_eq!(
            drained.rejected_cardinality, 5,
            "excess names are reported under the cardinality reason"
        );
        assert_eq!(drained.rejected_name_len, 0);
        assert_eq!(drained.value, MAX_GUEST_COUNTERS as u64);
    }

    /// Scenario: A guest submits a counter name larger than the host retention limit.
    /// Guarantees: The name is rejected and is not retained in host state, and
    /// the rejection is attributed to the name-length reason rather than the
    /// cardinality reason -- the two call for different operator responses.
    #[test]
    fn counter_add_rejects_oversized_name() {
        let mut host = HostState::new();
        counter_add(&mut host, &"x".repeat(MAX_GUEST_COUNTER_NAME_LEN + 1), 1);
        assert_eq!(host.guest_counter_count(), 0);
        assert_eq!(
            host.drain_counter_add_calls(),
            GuestCounterActivity {
                accepted: 0,
                rejected_name_len: 1,
                rejected_cardinality: 0,
                value: 0,
            }
        );
    }

    /// Scenario: A guest submits a `log` message larger than the host
    /// retention limit.
    /// Guarantees: The message is truncated to a valid UTF-8 prefix, the
    /// complete marked message remains within the byte limit, and the
    /// truncation is counted.
    #[test]
    fn log_truncates_oversized_message() {
        let mut message = "x".repeat(MAX_GUEST_LOG_MESSAGE_LEN + 1);
        assert!(truncate_guest_log_message(&mut message));
        assert_eq!(message.len(), MAX_GUEST_LOG_MESSAGE_LEN);
        assert!(message.ends_with(GUEST_LOG_TRUNCATION_MARKER));

        let mut host = HostState::new();
        log(
            &mut host,
            LogLevel::Info,
            "x".repeat(MAX_GUEST_LOG_MESSAGE_LEN + 1),
        );
        assert_eq!(host.drain_host_service_budget_calls(), (0, 1));
    }

    /// Scenario: A guest submits a `log` message that ends exactly on a
    /// multi-byte UTF-8 character straddling the truncation bound.
    /// Guarantees: Truncation backs off to the nearest char boundary rather
    /// than panicking, and the marked result remains within the byte limit.
    #[test]
    fn log_truncates_multi_byte_message_at_char_boundary() {
        let prefix_limit = MAX_GUEST_LOG_MESSAGE_LEN - GUEST_LOG_TRUNCATION_MARKER.len();
        let mut message = "a".repeat(prefix_limit - 1);
        message.push('\u{20AC}'); // 3-byte UTF-8 character (Euro sign).
        message.push_str(&"b".repeat(GUEST_LOG_TRUNCATION_MARKER.len()));
        assert!(message.len() > MAX_GUEST_LOG_MESSAGE_LEN);
        assert!(truncate_guest_log_message(&mut message));
        assert!(message.len() <= MAX_GUEST_LOG_MESSAGE_LEN);
        assert!(message.ends_with(GUEST_LOG_TRUNCATION_MARKER));
    }

    /// Scenario: A guest calls `host-services.counter-add` in a tight loop,
    /// far exceeding the shared token bucket's burst allowance.
    /// Guarantees: Calls beyond the burst are silently no-ops (neither
    /// create a counter nor emit telemetry) and are counted separately from
    /// the existing name/cardinality-based rejections; the rejection
    /// happens on every call once the bucket is empty (rather than only
    /// once), so a tight guest loop cannot generate unbounded telemetry
    /// volume no matter how many calls it makes in a single burst.
    #[test]
    fn host_service_calls_are_capped_by_burst_allowance() {
        let mut host = HostState::new();
        host.set_test_limiter_now(std::time::Instant::now());
        let total_calls = HOST_SERVICE_CALL_BURST as usize + 10;
        for i in 0..total_calls {
            counter_add(&mut host, &format!("counter.{i}"), 1);
        }
        // Only calls within the burst allowance reached the cardinality/
        // name-length checks at all; everything past it short-circuited on
        // the rate limiter first.
        let drained = host.drain_counter_add_calls();
        assert_eq!(drained.accepted as usize, MAX_GUEST_COUNTERS);
        assert_eq!(
            drained.rejected_cardinality as usize,
            HOST_SERVICE_CALL_BURST as usize - MAX_GUEST_COUNTERS
        );
        let (budget_rejected, _truncated) = host.drain_host_service_budget_calls();
        assert_eq!(budget_rejected, 10);
    }

    /// Scenario: The call-rate bucket is empty, but a guest keeps sending
    /// strings to both logging and counter imports.
    /// Guarantees: Rejected calls still consume the shared byte allowance;
    /// exactly 2 MiB fits and the next byte traps without updating counters.
    #[test]
    fn throttled_host_services_still_consume_copy_budget() {
        let mut host = HostState::new();
        host.set_test_limiter_now(std::time::Instant::now());
        host.host_service_rate_limiter.tokens = 0.0;
        let calls = GUEST_COPY_BYTE_BURST / MAX_GUEST_HOSTCALL_BYTES;
        for i in 0..calls {
            let text = "x".repeat(MAX_GUEST_HOSTCALL_BYTES);
            if i % 2 == 0 {
                host.log(LogLevel::Info, text).expect("within byte budget");
            } else {
                host.counter_add(text, 1).expect("within byte budget");
            }
        }
        assert_eq!(host.guest_copy_budget.remaining, 0);
        assert_eq!(host.host_service_calls_rejected, calls as u64);
        let error = host
            .counter_add("x".to_string(), 1)
            .expect_err("byte budget");
        assert!(
            error
                .to_string()
                .contains("string-copy byte budget exceeded")
        );
        assert_eq!(host.counter_add_calls, 0);
        assert_eq!(host.guest_counter_count(), 0);
    }

    /// Scenario: Expensive host work gives the byte bucket time to refill
    /// while the same guest entry point is still running.
    /// Guarantees: Refills cannot extend the non-refilling per-entry budget.
    #[test]
    fn copy_budget_per_entry_does_not_refill() {
        let mut host = HostState::new();
        let now = std::time::Instant::now();
        host.set_test_limiter_now(now);
        host.consume_guest_copy_budget(GUEST_COPY_BYTE_BURST)
            .unwrap();
        host.set_test_limiter_now(now + std::time::Duration::from_secs(10));
        assert!(host.consume_guest_copy_budget(1).is_err());
        host.begin_guest_call();
        host.consume_guest_copy_budget(GUEST_COPY_BYTE_BURST)
            .unwrap();
        assert!(host.consume_guest_copy_budget(1).is_err());
    }

    /// Scenario: Guest entries restart after exhausting the lifetime byte
    /// bucket, with a simulated one-second interval between calls.
    /// Guarantees: Entry resets do not refill the lifetime bucket; one second
    /// replenishes exactly 1 MiB and idle refills never exceed the 2 MiB burst.
    #[test]
    fn copy_budget_lifetime_rate_survives_entry_resets() {
        let mut host = HostState::new();
        let now = std::time::Instant::now();
        host.set_test_limiter_now(now);
        host.consume_guest_copy_budget(GUEST_COPY_BYTE_BURST)
            .unwrap();
        host.begin_guest_call();
        assert!(host.consume_guest_copy_budget(1).is_err());
        host.set_test_limiter_now(now + std::time::Duration::from_secs(1));
        host.consume_guest_copy_budget(1024 * 1024).unwrap();
        assert!(host.consume_guest_copy_budget(1).is_err());
        host.begin_guest_call();
        host.set_test_limiter_now(now + std::time::Duration::from_secs(100));
        host.consume_guest_copy_budget(GUEST_COPY_BYTE_BURST)
            .unwrap();
        host.begin_guest_call();
        assert!(host.consume_guest_copy_budget(1).is_err());
    }

    /// Scenario: A guest repeatedly fetches a large configuration blob.
    /// Guarantees: Config copies consume the same byte budget as incoming
    /// strings and an over-budget clone is rejected before allocating it.
    #[test]
    fn config_clones_consume_copy_budget() {
        let mut host = HostState::with_config("x".repeat(GUEST_COPY_BYTE_BURST));
        host.set_test_limiter_now(std::time::Instant::now());
        assert_eq!(host.get_config().unwrap().len(), GUEST_COPY_BYTE_BURST);
        assert!(host.get_config().is_err());
        assert!(host.counter_add("x".to_string(), 1).is_err());
    }

    /// Scenario: A filter's two string arguments exceed the remaining byte
    /// allowance even though either string alone would fit.
    /// Guarantees: Kernel strings share the copy budget and rejection occurs
    /// before consuming the input pdata or doing native Arrow work.
    #[test]
    fn kernel_strings_consume_shared_copy_budget() {
        let mut host = HostState::new();
        host.set_test_limiter_now(std::time::Instant::now());
        host.consume_guest_copy_budget(GUEST_COPY_BYTE_BURST - 1)
            .unwrap();
        let handle = host
            .table
            .push(HostPdata {
                otap_batch: batch_with_severity(&["ERROR"]),
            })
            .unwrap();
        let rep = handle.rep();
        let error = host
            .filter_by_attribute_eq(handle, AttrScope::Record, "k".to_string(), "v".to_string())
            .expect_err("both strings must be charged");
        assert!(
            error
                .to_string()
                .contains("string-copy byte budget exceeded")
        );
        assert_eq!(host.kernel_calls, 0);
        assert!(
            host.table
                .get(&Resource::<HostPdata>::new_borrow(rep))
                .is_ok()
        );
    }

    /// Scenario: A guest exhausts the shared token bucket's burst
    /// allowance, then (simulated via a synthetic clock, not a real sleep)
    /// enough wall-clock time passes for the sustained rate to refill a
    /// single token.
    /// Guarantees: Rate-limiting is a continuously-refilling budget scoped
    /// to the plugin instance's lifetime -- not a fixed allowance that
    /// resets only on an explicit per-call reset -- so a long-lived guest
    /// that throttles down to the sustained rate keeps making progress
    /// instead of being permanently cut off after one burst.
    #[test]
    fn rate_limiter_refills_over_simulated_time() {
        let mut limiter = HostServiceRateLimiter::new();
        let t0 = std::time::Instant::now();

        // Drain the full burst allowance at t0.
        for _ in 0..HOST_SERVICE_CALL_BURST as u64 {
            assert!(limiter.try_consume(t0));
        }
        assert!(
            !limiter.try_consume(t0),
            "bucket must be empty immediately after the burst is exhausted"
        );

        // Just under one token's worth of time at the sustained rate: still
        // empty.
        let almost_one_token =
            t0 + std::time::Duration::from_secs_f64(0.5 / HOST_SERVICE_CALL_RATE_PER_SEC);
        assert!(!limiter.try_consume(almost_one_token));

        // A bit more than one token's worth of time: exactly one call is
        // allowed, then the bucket is empty again.
        let just_over_one_token =
            t0 + std::time::Duration::from_secs_f64(1.5 / HOST_SERVICE_CALL_RATE_PER_SEC);
        assert!(limiter.try_consume(just_over_one_token));
        assert!(!limiter.try_consume(just_over_one_token));

        // A full second later: the sustained rate refills up to (but not
        // beyond) the burst cap.
        let one_second_later = t0 + std::time::Duration::from_secs(1);
        let mut allowed = 0;
        while limiter.try_consume(one_second_later) {
            allowed += 1;
        }
        assert!(
            (allowed as f64 - HOST_SERVICE_CALL_RATE_PER_SEC).abs() <= 1.0,
            "expected roughly one second's worth of sustained-rate tokens, got {allowed}"
        );
    }

    /// Scenario: `host-services.log` is called at every supported severity
    /// level.
    /// Guarantees: The call routes through the host's `otel_*` telemetry
    /// macros without panicking, for every `LogLevel` variant (this is a
    /// smoke test verifying the dispatch match is exhaustive and does not
    /// trap; the exact log record layout is exercised in the integration
    /// test where a real subscriber is not installed either, so this
    /// primarily guards against a match arm regression that would fail to
    /// compile or panic at runtime).
    #[test]
    fn log_dispatches_for_every_level() {
        let mut host = HostState::new();
        for level in [
            LogLevel::Trace,
            LogLevel::Debug,
            LogLevel::Info,
            LogLevel::Warn,
            LogLevel::Error,
        ] {
            log(&mut host, level, "hello from test".to_string());
        }
    }

    /// Scenario: Two plugin instances belonging to different pipeline nodes
    /// each emit guest log lines (at every severity, including the `trace`
    /// arm that routes through `otel_event!` rather than a level-specific
    /// macro) and one of them reports its guest counters.
    /// Guarantees: Every guest-forwarded record is routed under the
    /// `wasm_processor` component target and carries a `node` attribute
    /// naming the emitting instance. The host stamps `node` itself, from
    /// state the guest cannot reach, so guest telemetry is always
    /// attributable to a specific plugin node and a guest can neither forge
    /// nor suppress that attribution.
    #[test]
    fn guest_telemetry_is_attributed_to_its_node() {
        use std::sync::{Arc, Mutex};
        use tracing::field::{Field, Visit};
        use tracing_subscriber::layer::{Context, Layer};
        use tracing_subscriber::prelude::*;

        #[derive(Default)]
        struct Captured {
            records: Vec<(String, String)>,
        }

        #[derive(Clone, Default)]
        struct Capture {
            captured: Arc<Mutex<Captured>>,
        }

        struct NodeVisitor {
            node: Option<String>,
        }

        impl Visit for NodeVisitor {
            fn record_str(&mut self, field: &Field, value: &str) {
                if field.name() == "node" {
                    self.node = Some(value.to_string());
                }
            }
            fn record_debug(&mut self, _field: &Field, _value: &dyn core::fmt::Debug) {}
        }

        impl<S: tracing::Subscriber> Layer<S> for Capture {
            fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
                let mut visitor = NodeVisitor { node: None };
                event.record(&mut visitor);
                self.captured.lock().expect("capture lock").records.push((
                    event.metadata().target().to_string(),
                    visitor.node.unwrap_or_default(),
                ));
            }
        }

        let capture = Capture::default();
        let subscriber = tracing_subscriber::registry().with(capture.clone());
        tracing::subscriber::with_default(subscriber, || {
            let mut a = HostState::with_config_for_node(String::new(), "filter_a".to_string());
            for level in [
                LogLevel::Trace,
                LogLevel::Debug,
                LogLevel::Info,
                LogLevel::Warn,
                LogLevel::Error,
            ] {
                log(&mut a, level, "from a".to_string());
            }
            // `counter-add` itself emits nothing (it must stay off the pdata
            // hot path); the per-name totals surface on the metrics tick.
            counter_add(&mut a, "a.count", 1);
            a.report_guest_counters();

            let mut b = HostState::with_config_for_node(String::new(), "filter_b".to_string());
            log(&mut b, LogLevel::Error, "from b".to_string());
        });

        let records = capture
            .captured
            .lock()
            .expect("capture lock")
            .records
            .clone();
        for (target, _) in &records {
            assert_eq!(
                target, "otel.processor.wasm_processor",
                "guest telemetry must use the wasm_processor component target"
            );
        }
        let nodes: Vec<&str> = records.iter().map(|(_, n)| n.as_str()).collect();
        assert_eq!(
            nodes,
            vec![
                // five log levels, then the counter snapshot
                "filter_a", "filter_a", "filter_a", "filter_a", "filter_a", "filter_a", "filter_b"
            ],
            "every record must name the plugin instance that emitted it"
        );
    }

    /// Scenario: A guest calls `counter-add` repeatedly, then the host runs
    /// its metrics-collection step.
    /// Guarantees: `counter-add` emits no telemetry of its own -- so a plugin
    /// counting once per batch adds no log record to the pdata hot path --
    /// and the cumulative per-name totals are emitted once per collection
    /// tick instead.
    #[test]
    fn guest_counters_are_reported_on_collection_not_per_call() {
        use std::sync::{Arc, Mutex};
        use tracing_subscriber::layer::{Context, Layer};
        use tracing_subscriber::prelude::*;

        #[derive(Clone, Default)]
        struct Counting {
            events: Arc<Mutex<usize>>,
        }

        impl<S: tracing::Subscriber> Layer<S> for Counting {
            fn on_event(&self, _event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
                *self.events.lock().expect("event lock") += 1;
            }
        }

        let counting = Counting::default();
        let subscriber = tracing_subscriber::registry().with(counting.clone());
        tracing::subscriber::with_default(subscriber, || {
            let mut host = HostState::new();
            for _ in 0..100 {
                counter_add(&mut host, "guest.rows", 1);
            }
            counter_add(&mut host, "guest.batches", 1);
            assert_eq!(
                *counting.events.lock().expect("event lock"),
                0,
                "counter-add must not emit telemetry on the hot path"
            );

            host.report_guest_counters();
            assert_eq!(
                *counting.events.lock().expect("event lock"),
                2,
                "one record per distinct counter name, on the collection tick"
            );
            assert_eq!(host.guest_counter("guest.rows"), Some(100));
        });
    }

    /// Scenario: A dictionary-encoded column uses `LargeUtf8` values rather
    /// than the more common `Utf8`.
    /// Guarantees: The `LargeUtf8` branch in `dictionary_eq_mask_impl` is
    /// exercised and correctly identifies matching rows.
    #[test]
    fn handles_large_utf8_dictionary_values() {
        use arrow::array::{Int8Array, LargeStringArray};

        let keys = Int8Array::from(vec![0i8, 1, 0, 2]);
        let values = Arc::new(LargeStringArray::from(vec!["ERROR", "INFO", "WARN"]));
        let dict =
            DictionaryArray::try_new(keys, values as Arc<dyn Array>).expect("build LargeUtf8 dict");
        let mask = dictionary_string_eq_mask(&dict, "ERROR")
            .expect("LargeUtf8 dict mask should succeed")
            .expect("LargeUtf8 dict column should produce a mask");
        let kept: Vec<bool> = (0..mask.len()).map(|i| mask.value(i)).collect();
        assert_eq!(kept, vec![true, false, true, false]);
    }

    /// Scenario: A dictionary-encoded column uses `UInt32` integer keys.
    /// Guarantees: The `UInt32` dispatch arm in `dictionary_string_eq_mask` is
    /// exercised and matching rows are correctly identified.
    #[test]
    fn handles_uint32_key_dictionary() {
        use arrow::datatypes::UInt32Type;

        let dict: DictionaryArray<UInt32Type> =
            vec!["ERROR", "INFO", "ERROR", "WARN"].into_iter().collect();
        let mask = dictionary_string_eq_mask(&dict, "ERROR")
            .expect("UInt32-key dict mask should succeed")
            .expect("UInt32-key dict column should produce a mask");
        let kept: Vec<bool> = (0..mask.len()).map(|i| mask.value(i)).collect();
        assert_eq!(kept, vec![true, false, true, false]);
    }
}
